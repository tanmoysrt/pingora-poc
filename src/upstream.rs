use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct HostUpstreams {
    // host -> upstream_address -> (health, weight)
    hosts: Arc<RwLock<HashMap<String, (HashMap<String, bool>, usize)>>>,
}

#[derive(Serialize, Deserialize)]
pub struct UpstreamInfo {
    pub address: String,
    pub healthy: bool,
}

impl HostUpstreams {
    pub fn new() -> Self {
        Self {
            hosts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_upstream(&self, host: &str, upstream: &str) -> bool {
        let mut hosts = self.hosts.write().await;
        let (upstreams, _) = hosts
            .entry(host.to_string())
            .or_insert_with(|| (HashMap::new(), 0));

        if upstreams.contains_key(upstream) {
            false
        } else {
            upstreams.insert(upstream.to_string(), true);
            true
        }
    }

    pub async fn remove_upstream(&self, host: &str, upstream: &str) -> bool {
        let mut hosts = self.hosts.write().await;
        if let Some((upstreams, round_robin_index)) = hosts.get_mut(host) {
            let removed = upstreams.remove(upstream).is_some();
            if removed {
                // Reset round robin index if we removed an upstream
                // to avoid index out of bounds
                *round_robin_index = 0;
            }
            removed
        } else {
            false
        }
    }

    pub async fn get_healthy_upstream(&self, host: &str) -> Option<String> {
        let mut hosts = self.hosts.write().await;

        if let Some((upstreams, round_robin_index)) = hosts.get_mut(host) {
            // Get all healthy upstreams as a vector
            let healthy_upstreams: Vec<String> = upstreams
                .iter()
                .filter(|(_, healthy)| **healthy)
                .map(|(addr, _)| addr.clone())
                .collect();

            if healthy_upstreams.is_empty() {
                return None;
            }

            // Round robin selection
            let selected_upstream =
                healthy_upstreams[*round_robin_index % healthy_upstreams.len()].clone();

            // Increment round robin index
            *round_robin_index = (*round_robin_index + 1) % healthy_upstreams.len();

            Some(selected_upstream)
        } else {
            None
        }
    }
}
