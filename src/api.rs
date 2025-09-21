use crate::upstream::*;
use serde::Serialize;

use axum::{
    Router,
    extract::Path,
    response::Json,
    routing::{delete, get, post},
};

#[derive(Serialize)]
pub struct ApiResponse<T> {
    success: bool,
    message: String,
    data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(message: &str, data: Option<T>) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            data: None,
        }
    }
}

async fn add_upstream_to_host(
    Path((host, upstream)): Path<(String, String)>,
    axum::extract::State(upstreams): axum::extract::State<HostUpstreams>,
) -> Json<ApiResponse<UpstreamInfo>> {
    let added = upstreams.add_upstream(&host, &upstream).await;
    if added {
        let info = UpstreamInfo {
            address: upstream.clone(),
            healthy: true,
        };
        Json(ApiResponse::success(
            &format!("Upstream '{}' added to host '{}'", upstream, host),
            Some(info),
        ))
    } else {
        Json(ApiResponse::error(&format!(
            "Upstream '{}' already exists for host '{}'",
            upstream, host
        )))
    }
}

async fn remove_upstream_from_host(
    Path((host, upstream)): Path<(String, String)>,
    axum::extract::State(upstreams): axum::extract::State<HostUpstreams>,
) -> Json<ApiResponse<()>> {
    let removed = upstreams.remove_upstream(&host, &upstream).await;
    if removed {
        Json(ApiResponse::success(
            &format!("Upstream '{}' removed from host '{}'", upstream, host),
            None,
        ))
    } else {
        Json(ApiResponse::error(&format!(
            "Upstream '{}' not found for host '{}'",
            upstream, host
        )))
    }
}

pub fn start_api_server(upstreams: HostUpstreams, port: u16) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let app = Router::new()
                .route("/", get(|| async { "Hello World" }))
                .route(
                    "/hosts/{host}/upstreams/{upstream}",
                    post(add_upstream_to_host),
                )
                .route(
                    "/hosts/{host}/upstreams/{upstream}",
                    delete(remove_upstream_from_host),
                )
                .with_state(upstreams);

            let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
                .await
                .unwrap();
            println!("Management API running on port {}", port);
            axum::serve(listener, app).await.unwrap();
        });
    });
}
