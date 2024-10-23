/*
Copyright The Kubernetes Authors.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

// Code generated by client-gen. DO NOT EDIT.

package v1beta1

import (
	"context"

	v1beta1 "github.com/linkerd/linkerd2/controller/gen/apis/server/v1beta1"
	scheme "github.com/linkerd/linkerd2/controller/gen/client/clientset/versioned/scheme"
	v1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	types "k8s.io/apimachinery/pkg/types"
	watch "k8s.io/apimachinery/pkg/watch"
	gentype "k8s.io/client-go/gentype"
)

// ServersGetter has a method to return a ServerInterface.
// A group's client should implement this interface.
type ServersGetter interface {
	Servers(namespace string) ServerInterface
}

// ServerInterface has methods to work with Server resources.
type ServerInterface interface {
	Create(ctx context.Context, server *v1beta1.Server, opts v1.CreateOptions) (*v1beta1.Server, error)
	Update(ctx context.Context, server *v1beta1.Server, opts v1.UpdateOptions) (*v1beta1.Server, error)
	Delete(ctx context.Context, name string, opts v1.DeleteOptions) error
	DeleteCollection(ctx context.Context, opts v1.DeleteOptions, listOpts v1.ListOptions) error
	Get(ctx context.Context, name string, opts v1.GetOptions) (*v1beta1.Server, error)
	List(ctx context.Context, opts v1.ListOptions) (*v1beta1.ServerList, error)
	Watch(ctx context.Context, opts v1.ListOptions) (watch.Interface, error)
	Patch(ctx context.Context, name string, pt types.PatchType, data []byte, opts v1.PatchOptions, subresources ...string) (result *v1beta1.Server, err error)
	ServerExpansion
}

// servers implements ServerInterface
type servers struct {
	*gentype.ClientWithList[*v1beta1.Server, *v1beta1.ServerList]
}

// newServers returns a Servers
func newServers(c *ServerV1beta1Client, namespace string) *servers {
	return &servers{
		gentype.NewClientWithList[*v1beta1.Server, *v1beta1.ServerList](
			"servers",
			c.RESTClient(),
			scheme.ParameterCodec,
			namespace,
			func() *v1beta1.Server { return &v1beta1.Server{} },
			func() *v1beta1.ServerList { return &v1beta1.ServerList{} }),
	}
}
