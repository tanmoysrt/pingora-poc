use pingora::prelude::*;

mod proxy;
use crate::proxy::*;
mod upstream;
use crate::upstream::*;
mod api;
use crate::api::*;

fn main() {
    let host_upstreams = HostUpstreams::new();

    let mut pingora_service = Server::new(None).unwrap();
    pingora_service.bootstrap();

    // Create load balancer with host-based upstreams
    let lb = LB::new(host_upstreams.clone());
    let mut lb_service = http_proxy_service(&pingora_service.configuration, lb);
    lb_service.add_tcp("0.0.0.0:3000");
    pingora_service.add_service(lb_service);

    // Start api server in a separate thread
    start_api_server(host_upstreams, 4000);

    // Start pingora service
    pingora_service.run_forever();
}
