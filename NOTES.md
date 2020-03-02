## edge-20.2.3

This release introduces the first optional add-on `tracing`, added through the
new add-on model!

The existing optional `tracing` components Jaeger and OpenCensus can now be
installed as add-on components.

There will be more information to come about the new add-on model, but please
refer to the details of [#3955](https://github.com/linkerd/linkerd2/pull/3955) for how to get started.

* CLI
  * Added the `linkerd diagnostics` command to get metrics only from the
    control plane, excluding metrics from the data plane proxies (thanks
    @srv-twry!)
  * Added the `linkerd install --prometheus-image` option for installing a
    custom Prometheus image (thanks @christyjacob4!)
  * Fixed an issue with `linkerd upgrade` where changes to the `Namespace`
    object were ignored (thanks @supra08!)
* Controller
  * Added the `tracing` add-on which installs Jaeger and OpenCensus as add-on
    components (thanks @Pothulapati!!)
* Proxy
  * Increased the inbound router's default capacity from 100 to 10k to
    accommodate environments that have a high cardinality of virtual hosts
    served by a single pod
* Web UI
  * Fixed styling in the CallToAction banner (thanks @aliariff!)

