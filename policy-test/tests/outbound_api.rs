use std::{collections::BTreeMap, time::Duration};

use futures::prelude::*;
use kube::ResourceExt;
use linkerd_policy_controller_k8s_api as k8s;
use linkerd_policy_test::{
    assert_default_accrual_backoff, create, create_annotated_service, create_cluster_scoped,
    create_opaque_service, create_service, delete_cluster_scoped, grpc, mk_service, with_temp_ns,
};
use maplit::{btreemap, convert_args};
use tokio::time;

#[tokio::test(flavor = "current_thread")]
async fn service_does_not_exist() {
    with_temp_ns(|client, ns| async move {
        // Build a service but don't apply it to the cluster.
        let mut svc = mk_service(&ns, "my-svc", 4191);
        // Give it a bogus cluster ip.
        svc.spec.as_mut().unwrap().cluster_ip = Some("1.1.1.1".to_string());

        let mut policy_api = grpc::OutboundPolicyClient::port_forwarded(&client).await;
        let rsp = policy_api.watch(&ns, &svc, 4191).await;

        assert!(rsp.is_err());
        assert_eq!(rsp.err().unwrap().code(), tonic::Code::NotFound);
    })
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn service_with_no_http_routes() {
    with_temp_ns(|client, ns| async move {
        // Create a service
        let svc = create_service(&client, &ns, "my-svc", 4191).await;

        let mut rx = retry_watch_outbound_policy(&client, &ns, &svc).await;
        let config = rx
            .next()
            .await
            .expect("watch must not fail")
            .expect("watch must return an initial config");
        tracing::trace!(?config);

        // There should be a default route.
        detect_http_routes(&config, |routes| {
            let route = assert_singleton(routes);
            assert_route_is_default(route, &svc, 4191);
        });
    })
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn service_with_http_route_without_rules() {
    with_temp_ns(|client, ns| async move {
        // Create a service
        let svc = create_service(&client, &ns, "my-svc", 4191).await;

        let mut rx = retry_watch_outbound_policy(&client, &ns, &svc).await;
        let config = rx
            .next()
            .await
            .expect("watch must not fail")
            .expect("watch must return an initial config");
        tracing::trace!(?config);

        // There should be a default route.
        detect_http_routes(&config, |routes| {
            let route = assert_singleton(routes);
            assert_route_is_default(route, &svc, 4191);
        });

        let _route = create(&client, mk_empty_http_route(&ns, "foo-route", &svc, 4191)).await;

        let mut rx = retry_watch_outbound_policy(&client, &ns, &svc).await;
        let config = rx
            .next()
            .await
            .expect("watch must not fail")
            .expect("watch must return an updated config");
        tracing::trace!(?config);

        // There should be a route with no rules.
        detect_http_routes(&config, |routes| {
            let route = assert_singleton(routes);
            assert_eq!(route.rules.len(), 0);
        });
    })
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn service_with_http_routes_without_backends() {
    with_temp_ns(|client, ns| async move {
        // Create a service
        let svc = create_service(&client, &ns, "my-svc", 4191).await;

        let mut rx = retry_watch_outbound_policy(&client, &ns, &svc).await;
        let config = rx
            .next()
            .await
            .expect("watch must not fail")
            .expect("watch must return an initial config");
        tracing::trace!(?config);

        // There should be a default route.
        detect_http_routes(&config, |routes| {
            let route = assert_singleton(routes);
            assert_route_is_default(route, &svc, 4191);
        });

        let _route = create(
            &client,
            mk_http_route(&ns, "foo-route", &svc, 4191, None, None),
        )
        .await;

        let config = rx
            .next()
            .await
            .expect("watch must not fail")
            .expect("watch must return an updated config");
        tracing::trace!(?config);

        // There should be a route with the logical backend.
        detect_http_routes(&config, |routes| {
            let route = assert_singleton(routes);
            let backends = route_backends_first_available(route);
            let backend = assert_singleton(backends);
            assert_backend_matches_service(backend, &svc, 4191);
        });
    })
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn service_with_http_routes_with_backend() {
    with_temp_ns(|client, ns| async move {
        // Create a service
        let svc = create_service(&client, &ns, "my-svc", 4191).await;

        let mut rx = retry_watch_outbound_policy(&client, &ns, &svc).await;
        let config = rx
            .next()
            .await
            .expect("watch must not fail")
            .expect("watch must return an initial config");
        tracing::trace!(?config);

        // There should be a default route.
        detect_http_routes(&config, |routes| {
            let route = assert_singleton(routes);
            assert_route_is_default(route, &svc, 4191);
        });

        let backend_name = "backend";
        let backend_svc = create_service(&client, &ns, backend_name, 8888).await;
        let backends = [backend_name];
        let _route = create(
            &client,
            mk_http_route(&ns, "foo-route", &svc, 4191, Some(&backends), None),
        )
        .await;

        let config = rx
            .next()
            .await
            .expect("watch must not fail")
            .expect("watch must return an updated config");
        tracing::trace!(?config);

        // There should be a route with a backend with no filters.
        detect_http_routes(&config, |routes| {
            let route = assert_singleton(routes);
            let backends = route_backends_random_available(route);
            let backend = assert_singleton(backends);
            assert_backend_matches_service(backend.backend.as_ref().unwrap(), &backend_svc, 8888);
            let filters = &backend.backend.as_ref().unwrap().filters;
            assert_eq!(filters.len(), 0);
        });
    })
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn service_with_http_routes_with_cross_namespace_backend() {
    with_temp_ns(|client, ns| async move {
        // Create a service
        let svc = create_service(&client, &ns, "my-svc", 4191).await;

        let mut rx = retry_watch_outbound_policy(&client, &ns, &svc).await;
        let config = rx
            .next()
            .await
            .expect("watch must not fail")
            .expect("watch must return an initial config");
        tracing::trace!(?config);

        // There should be a default route.
        detect_http_routes(&config, |routes| {
            let route = assert_singleton(routes);
            assert_route_is_default(route, &svc, 4191);
        });

        let backend_ns_name = format!("{}-backend", ns);
        let backend_ns = create_cluster_scoped(
            &client,
            k8s::Namespace {
                metadata: k8s::ObjectMeta {
                    name: Some(backend_ns_name.clone()),
                    labels: Some(convert_args!(btreemap!(
                        "linkerd-policy-test" => std::thread::current().name().unwrap_or(""),
                    ))),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .await;
        let backend_name = "backend";
        let backend_svc = create_service(&client, &backend_ns_name, backend_name, 8888).await;
        let backends = [backend_name];
        let _route = create(
            &client,
            mk_http_route(
                &ns,
                "foo-route",
                &svc,
                4191,
                Some(&backends),
                Some(backend_ns_name),
            ),
        )
        .await;

        let config = rx
            .next()
            .await
            .expect("watch must not fail")
            .expect("watch must return an updated config");
        tracing::trace!(?config);

        // There should be a route with a backend with no filters.
        detect_http_routes(&config, |routes| {
            let route = assert_singleton(routes);
            let backends = route_backends_random_available(route);
            let backend = assert_singleton(backends);
            assert_backend_matches_service(backend.backend.as_ref().unwrap(), &backend_svc, 8888);
            let filters = &backend.backend.as_ref().unwrap().filters;
            assert_eq!(filters.len(), 0);
        });

        delete_cluster_scoped(&client, backend_ns).await
    })
    .await;
}

// TODO: Test fails until handling of invalid backends is implemented.
#[tokio::test(flavor = "current_thread")]
async fn service_with_http_routes_with_invalid_backend() {
    with_temp_ns(|client, ns| async move {
        // Create a service
        let svc = create_service(&client, &ns, "my-svc", 4191).await;

        let mut rx = retry_watch_outbound_policy(&client, &ns, &svc).await;
        let config = rx
            .next()
            .await
            .expect("watch must not fail")
            .expect("watch must return an initial config");
        tracing::trace!(?config);

        // There should be a default route.
        detect_http_routes(&config, |routes| {
            let route = assert_singleton(routes);
            assert_route_is_default(route, &svc, 4191);
        });

        let backends = ["invalid-backend"];
        let _route = create(
            &client,
            mk_http_route(&ns, "foo-route", &svc, 4191, Some(&backends), None),
        )
        .await;

        let config = rx
            .next()
            .await
            .expect("watch must not fail")
            .expect("watch must return an updated config");
        tracing::trace!(?config);

        // There should be a route with a backend.
        detect_http_routes(&config, |routes| {
            let route = assert_singleton(routes);
            let backends = route_backends_random_available(route);
            let backend = assert_singleton(backends);
            assert_backend_has_failure_filter(backend);
        });
    })
    .await;
}

// TODO: Investigate why the policy controller is only returning one route in this
// case instead of two.
#[tokio::test(flavor = "current_thread")]
async fn service_with_multiple_http_routes() {
    with_temp_ns(|client, ns| async move {
        // Create a service
        let svc = create_service(&client, &ns, "my-svc", 4191).await;

        let mut rx = retry_watch_outbound_policy(&client, &ns, &svc).await;
        let config = rx
            .next()
            .await
            .expect("watch must not fail")
            .expect("watch must return an initial config");
        tracing::trace!(?config);

        // There should be a default route.
        detect_http_routes(&config, |routes| {
            let route = assert_singleton(routes);
            assert_route_is_default(route, &svc, 4191);
        });

        // Routes should be returned in sorted order by creation timestamp then
        // name. To ensure that this test isn't timing dependant, routes should
        // be created in alphabetical order.
        let _a_route = create(
            &client,
            mk_http_route(&ns, "a-route", &svc, 4191, None, None),
        )
        .await;

        // First route update.
        let config = rx
            .next()
            .await
            .expect("watch must not fail")
            .expect("watch must return an updated config");
        tracing::trace!(?config);

        let _b_route = create(
            &client,
            mk_http_route(&ns, "b-route", &svc, 4191, None, None),
        )
        .await;

        // Second route update.
        let config = rx
            .next()
            .await
            .expect("watch must not fail")
            .expect("watch must return an updated config");
        tracing::trace!(?config);

        // There should be 2 routes, returned in order.
        detect_http_routes(&config, |routes| {
            assert_eq!(routes.len(), 2);
            assert_eq!(route_name(&routes[0]), "a-route");
            assert_eq!(route_name(&routes[1]), "b-route");
        });
    })
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn service_with_consecutive_failure_accrual() {
    with_temp_ns(|client, ns| async move {
        let svc = create_annotated_service(
            &client,
            &ns,
            "consecutive-accrual-svc",
            80,
            BTreeMap::from([
                (
                    "balancer.linkerd.io/failure-accrual".to_string(),
                    "consecutive".to_string(),
                ),
                (
                    "balancer.linkerd.io/failure-accrual-consecutive-max-failures".to_string(),
                    "8".to_string(),
                ),
                (
                    "balancer.linkerd.io/failure-accrual-consecutive-min-penalty".to_string(),
                    "10s".to_string(),
                ),
                (
                    "balancer.linkerd.io/failure-accrual-consecutive-max-penalty".to_string(),
                    "10m".to_string(),
                ),
                (
                    "balancer.linkerd.io/failure-accrual-consecutive-jitter-ratio".to_string(),
                    "1.0".to_string(),
                ),
            ]),
        )
        .await;

        let mut rx = retry_watch_outbound_policy(&client, &ns, &svc).await;
        let config = rx
            .next()
            .await
            .expect("watch must not fail")
            .expect("watch must return an initial config");
        tracing::trace!(?config);

        detect_failure_accrual(&config, |accrual| {
            let consecutive = failure_accrual_consecutive(accrual);
            assert_eq!(8, consecutive.max_failures);
            assert_eq!(
                &grpc::outbound::ExponentialBackoff {
                    min_backoff: Some(Duration::from_secs(10).try_into().unwrap()),
                    max_backoff: Some(Duration::from_secs(600).try_into().unwrap()),
                    jitter_ratio: 1.0_f32,
                },
                consecutive
                    .backoff
                    .as_ref()
                    .expect("backoff must be configured")
            );
        });
    })
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn service_with_consecutive_failure_accrual_defaults() {
    with_temp_ns(|client, ns| async move {
        // Create a service configured to do consecutive failure accrual, but
        // with no additional configuration
        let svc = create_annotated_service(
            &client,
            &ns,
            "default-accrual-svc",
            80,
            BTreeMap::from([(
                "balancer.linkerd.io/failure-accrual".to_string(),
                "consecutive".to_string(),
            )]),
        )
        .await;

        let mut rx = retry_watch_outbound_policy(&client, &ns, &svc).await;
        let config = rx
            .next()
            .await
            .expect("watch must not fail")
            .expect("watch must return an initial config");
        tracing::trace!(?config);

        // Expect default max_failures and default backoff
        detect_failure_accrual(&config, |accrual| {
            let consecutive = failure_accrual_consecutive(accrual);
            assert_eq!(7, consecutive.max_failures);
            assert_default_accrual_backoff!(consecutive
                .backoff
                .as_ref()
                .expect("backoff must be configured"));
        });

        // Create a service configured to do consecutive failure accrual with
        // max number of failures and with default backoff
        let svc = create_annotated_service(
            &client,
            &ns,
            "no-backoff-svc",
            80,
            BTreeMap::from([
                (
                    "balancer.linkerd.io/failure-accrual".to_string(),
                    "consecutive".to_string(),
                ),
                (
                    "balancer.linkerd.io/failure-accrual-consecutive-max-failures".to_string(),
                    "8".to_string(),
                ),
            ]),
        )
        .await;

        let mut rx = retry_watch_outbound_policy(&client, &ns, &svc).await;
        let config = rx
            .next()
            .await
            .expect("watch must not fail")
            .expect("watch must return an initial config");
        tracing::trace!(?config);

        // Expect default backoff and overridden max_failures
        detect_failure_accrual(&config, |accrual| {
            let consecutive = failure_accrual_consecutive(accrual);
            assert_eq!(8, consecutive.max_failures);
            assert_default_accrual_backoff!(consecutive
                .backoff
                .as_ref()
                .expect("backoff must be configured"));
        });

        // Create a service configured to do consecutive failure accrual with
        // only the jitter ratio configured in the backoff
        let svc = create_annotated_service(
            &client,
            &ns,
            "only-jitter-svc",
            80,
            BTreeMap::from([
                (
                    "balancer.linkerd.io/failure-accrual".to_string(),
                    "consecutive".to_string(),
                ),
                (
                    "balancer.linkerd.io/failure-accrual-consecutive-jitter-ratio".to_string(),
                    "1.0".to_string(),
                ),
            ]),
        )
        .await;

        let mut rx = retry_watch_outbound_policy(&client, &ns, &svc).await;
        let config = rx
            .next()
            .await
            .expect("watch must not fail")
            .expect("watch must return an initial config");
        tracing::trace!(?config);

        // Expect defaults for everything except for the jitter ratio
        detect_failure_accrual(&config, |accrual| {
            let consecutive = failure_accrual_consecutive(accrual);
            assert_eq!(7, consecutive.max_failures);
            assert_eq!(
                &grpc::outbound::ExponentialBackoff {
                    min_backoff: Some(Duration::from_secs(1).try_into().unwrap()),
                    max_backoff: Some(Duration::from_secs(60).try_into().unwrap()),
                    jitter_ratio: 1.0_f32,
                },
                consecutive
                    .backoff
                    .as_ref()
                    .expect("backoff must be configured")
            );
        });
    })
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn service_with_default_failure_accrual() {
    with_temp_ns(|client, ns| async move {
        // Default config for Service, no failure accrual
        let svc = create_service(&client, &ns, "default-failure-accrual", 80).await;

        let mut rx = retry_watch_outbound_policy(&client, &ns, &svc).await;
        let config = rx
            .next()
            .await
            .expect("watch must not fail")
            .expect("watch must return an initial config");
        tracing::trace!(?config);

        // Expect failure accrual config to be default (no failure accrual)
        detect_failure_accrual(&config, |accrual| {
            assert!(
                accrual.is_none(),
                "consecutive failure accrual should not be configured for service"
            );
        });

        // Create Service with consecutive failure accrual config for
        // max_failures but no mode
        let svc = create_annotated_service(
            &client,
            &ns,
            "default-max-failure-svc",
            80,
            BTreeMap::from([(
                "balancer.linkerd.io/failure-accrual-consecutive-max-failures".to_string(),
                "8".to_string(),
            )]),
        )
        .await;

        let mut rx = retry_watch_outbound_policy(&client, &ns, &svc).await;
        let config = rx
            .next()
            .await
            .expect("watch must not fail")
            .expect("watch must return an initial config");
        tracing::trace!(?config);

        // Expect failure accrual config to be default (no failure accrual)
        detect_failure_accrual(&config, |accrual| {
            assert!(
                accrual.is_none(),
                "consecutive failure accrual should not be configured for service"
            )
        });
    })
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn opaque_service() {
    with_temp_ns(|client, ns| async move {
        // Create a service
        let svc = create_opaque_service(&client, &ns, "my-svc", 4191).await;

        let mut rx = retry_watch_outbound_policy(&client, &ns, &svc).await;
        let config = rx
            .next()
            .await
            .expect("watch must not fail")
            .expect("watch must return an initial config");
        tracing::trace!(?config);

        // Proxy protocol should be opaque.
        match config.protocol.unwrap().kind.unwrap() {
            grpc::outbound::proxy_protocol::Kind::Opaque(_) => {}
            _ => panic!("proxy protocol must be Opaque"),
        };
    })
    .await;
}

/* Helpers */

async fn retry_watch_outbound_policy(
    client: &kube::Client,
    ns: &str,
    svc: &k8s::Service,
) -> tonic::Streaming<grpc::outbound::OutboundPolicy> {
    // Port-forward to the control plane and start watching the service's
    // outbound policy.
    let mut policy_api = grpc::OutboundPolicyClient::port_forwarded(client).await;
    loop {
        match policy_api.watch(ns, svc, 4191).await {
            Ok(rx) => return rx,
            Err(error) => {
                tracing::error!(
                    ?error,
                    ns,
                    svc = svc.name_unchecked(),
                    "failed to watch outbound policy for port 4191"
                );
                time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

fn mk_http_route(
    ns: &str,
    name: &str,
    svc: &k8s::Service,
    port: u16,
    backends: Option<&[&str]>,
    backends_ns: Option<String>,
) -> k8s::policy::HttpRoute {
    use k8s::policy::httproute as api;
    let backend_refs = backends.map(|names| {
        names
            .iter()
            .map(|name| api::HttpBackendRef {
                backend_ref: Some(k8s_gateway_api::BackendRef {
                    weight: None,
                    inner: k8s_gateway_api::BackendObjectReference {
                        name: name.to_string(),
                        port: Some(8888),
                        group: None,
                        kind: None,
                        namespace: backends_ns.clone(),
                    },
                }),
                filters: None,
            })
            .collect()
    });
    api::HttpRoute {
        metadata: kube::api::ObjectMeta {
            namespace: Some(ns.to_string()),
            name: Some(name.to_string()),
            ..Default::default()
        },
        spec: api::HttpRouteSpec {
            inner: api::CommonRouteSpec {
                parent_refs: Some(vec![api::ParentReference {
                    group: Some("core".to_string()),
                    kind: Some("Service".to_string()),
                    namespace: svc.namespace(),
                    name: svc.name_unchecked(),
                    section_name: None,
                    port: Some(port),
                }]),
            },
            hostnames: None,
            rules: Some(vec![api::HttpRouteRule {
                matches: Some(vec![api::HttpRouteMatch {
                    path: Some(api::HttpPathMatch::Exact {
                        value: "/foo".to_string(),
                    }),
                    headers: None,
                    query_params: None,
                    method: Some("GET".to_string()),
                }]),
                filters: None,
                backend_refs,
            }]),
        },
        status: None,
    }
}

fn mk_empty_http_route(
    ns: &str,
    name: &str,
    svc: &k8s::Service,
    port: u16,
) -> k8s::policy::HttpRoute {
    use k8s::policy::httproute as api;
    api::HttpRoute {
        metadata: kube::api::ObjectMeta {
            namespace: Some(ns.to_string()),
            name: Some(name.to_string()),
            ..Default::default()
        },
        spec: api::HttpRouteSpec {
            inner: api::CommonRouteSpec {
                parent_refs: Some(vec![api::ParentReference {
                    group: Some("core".to_string()),
                    kind: Some("Service".to_string()),
                    namespace: svc.namespace(),
                    name: svc.name_unchecked(),
                    section_name: None,
                    port: Some(port),
                }]),
            },
            hostnames: None,
            rules: Some(vec![]),
        },
        status: None,
    }
}

// detect_http_routes asserts that the given outbound policy has a proxy protcol
// of "Detect" and then invokes the given function with the Http1 and Http2
// routes from the Detect.
#[track_caller]
fn detect_http_routes<F>(config: &grpc::outbound::OutboundPolicy, f: F)
where
    F: Fn(&[grpc::outbound::HttpRoute]),
{
    let kind = config
        .protocol
        .as_ref()
        .expect("must have proxy protocol")
        .kind
        .as_ref()
        .expect("must have kind");
    if let grpc::outbound::proxy_protocol::Kind::Detect(grpc::outbound::proxy_protocol::Detect {
        opaque: _,
        timeout: _,
        http1,
        http2,
    }) = kind
    {
        let http1 = http1
            .as_ref()
            .expect("proxy protocol must have http1 field");
        let http2 = http2
            .as_ref()
            .expect("proxy protocol must have http2 field");
        f(&http1.routes);
        f(&http2.routes);
    } else {
        panic!("proxy protocol must be Detect; actually got:\n{kind:#?}")
    }
}

#[track_caller]
fn detect_failure_accrual<F>(config: &grpc::outbound::OutboundPolicy, f: F)
where
    F: Fn(Option<&grpc::outbound::FailureAccrual>),
{
    let kind = config
        .protocol
        .as_ref()
        .expect("must have proxy protocol")
        .kind
        .as_ref()
        .expect("must have kind");
    if let grpc::outbound::proxy_protocol::Kind::Detect(grpc::outbound::proxy_protocol::Detect {
        opaque: _,
        timeout: _,
        http1,
        http2,
    }) = kind
    {
        let http1 = http1
            .as_ref()
            .expect("proxy protocol must have http1 field");
        let http2 = http2
            .as_ref()
            .expect("proxy protocol must have http2 field");
        f(http1.failure_accrual.as_ref());
        f(http2.failure_accrual.as_ref());
    } else {
        panic!("proxy protocol must be Detect; actually got:\n{kind:#?}")
    }
}

#[track_caller]
fn failure_accrual_consecutive(
    accrual: Option<&grpc::outbound::FailureAccrual>,
) -> &grpc::outbound::failure_accrual::ConsecutiveFailures {
    assert!(
        accrual.is_some(),
        "failure accrual must be configured for service"
    );
    let kind = accrual
        .unwrap()
        .kind
        .as_ref()
        .expect("failure accrual must have kind");
    let grpc::outbound::failure_accrual::Kind::ConsecutiveFailures(accrual) = kind;
    accrual
}

#[track_caller]
fn route_backends_first_available(
    route: &grpc::outbound::HttpRoute,
) -> &[grpc::outbound::http_route::RouteBackend] {
    let kind = assert_singleton(&route.rules)
        .backends
        .as_ref()
        .expect("Rule must have backends")
        .kind
        .as_ref()
        .expect("Backend must have kind");
    match kind {
        grpc::outbound::http_route::distribution::Kind::FirstAvailable(fa) => &fa.backends,
        _ => panic!("Distribution must be FirstAvailable"),
    }
}

#[track_caller]
fn route_backends_random_available(
    route: &grpc::outbound::HttpRoute,
) -> &[grpc::outbound::http_route::WeightedRouteBackend] {
    let kind = assert_singleton(&route.rules)
        .backends
        .as_ref()
        .expect("Rule must have backends")
        .kind
        .as_ref()
        .expect("Backend must have kind");
    match kind {
        grpc::outbound::http_route::distribution::Kind::RandomAvailable(dist) => &dist.backends,
        _ => panic!("Distribution must be RandomAvailable"),
    }
}

#[track_caller]
fn route_name(route: &grpc::outbound::HttpRoute) -> &str {
    match route.metadata.as_ref().unwrap().kind.as_ref().unwrap() {
        grpc::meta::metadata::Kind::Resource(grpc::meta::Resource { ref name, .. }) => name,
        _ => panic!("route must be a resource kind"),
    }
}

#[track_caller]
fn assert_backend_has_failure_filter(backend: &grpc::outbound::http_route::WeightedRouteBackend) {
    let filter = assert_singleton(&backend.backend.as_ref().unwrap().filters);
    match filter.kind.as_ref().unwrap() {
        grpc::outbound::http_route::filter::Kind::FailureInjector(_) => {}
        _ => panic!("backend must have FailureInjector filter"),
    };
}

#[track_caller]
fn assert_route_is_default(route: &grpc::outbound::HttpRoute, svc: &k8s::Service, port: u16) {
    let backends = route_backends_first_available(route);
    let backend = assert_singleton(backends);
    assert_backend_matches_service(backend, svc, port);

    let rule = assert_singleton(&route.rules);
    let route_match = assert_singleton(&rule.matches);
    let path_match = route_match.path.as_ref().unwrap().kind.as_ref().unwrap();
    assert_eq!(
        *path_match,
        grpc::http_route::path_match::Kind::Prefix("/".to_string())
    );
}

#[track_caller]
fn assert_backend_matches_service(
    backend: &grpc::outbound::http_route::RouteBackend,
    svc: &k8s::Service,
    port: u16,
) {
    let kind = backend.backend.as_ref().unwrap().kind.as_ref().unwrap();
    let dst = match kind {
        grpc::outbound::backend::Kind::Balancer(balance) => {
            let kind = balance.discovery.as_ref().unwrap().kind.as_ref().unwrap();
            match kind {
                grpc::outbound::backend::endpoint_discovery::Kind::Dst(dst) => &dst.path,
            }
        }
        grpc::outbound::backend::Kind::Forward(_) => {
            panic!("default route backend must be Balancer")
        }
    };
    assert_eq!(
        *dst,
        format!(
            "{}.{}.svc.{}:{}",
            svc.name_unchecked(),
            svc.namespace().unwrap(),
            "cluster.local",
            port
        )
    );
}

#[track_caller]
fn assert_singleton<T>(ts: &[T]) -> &T {
    assert_eq!(ts.len(), 1);
    ts.get(0).unwrap()
}