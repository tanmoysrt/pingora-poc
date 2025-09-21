use crate::upstream::*;
use async_trait::async_trait;
use pingora::prelude::*;

pub struct LB {
    upstreams: HostUpstreams,
}

impl LB {
    pub fn new(upstreams: HostUpstreams) -> Self {
        Self { upstreams }
    }
}

#[async_trait]
impl ProxyHttp for LB {
    type CTX = ();

    fn new_ctx(&self) -> () {
        ()
    }

    async fn upstream_peer(&self, session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let host = session
            .req_header()
            .headers
            .get("Host")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("default");

        let upstream = self.upstreams.get_healthy_upstream(host).await;

        match upstream {
            Some(addr) => {
                println!("upstream for host '{}' -> {}", host, addr);
                let peer = Box::new(HttpPeer::new(addr, false, host.to_string()));
                Ok(peer)
            }
            None => {
                println!("No upstreams for host: {}", host);
                Err(Error::new(ErrorType::ConnectError))
            }
        }
    }

    async fn upstream_request_filter(
        &self,
        session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        let client_ip = session.client_addr();
        match client_ip {
            Some(ip) => upstream_request.insert_header("X-Forwarded-For", ip.to_string()),
            None => {
                println!("Could not determine client IP");
                Ok(())
            }
        }
    }
}
