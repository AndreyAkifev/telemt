use crate::config::{UpstreamConfig, UpstreamType};
use crate::error::{ProxyError, Result};

fn encode_component(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

fn upstream_to_socks_url(upstream: &UpstreamConfig) -> Option<String> {
    match &upstream.upstream_type {
        UpstreamType::Socks5 {
            address,
            username,
            password,
            ..
        } => {
            let auth = match (username.as_deref(), password.as_deref()) {
                (Some(u), Some(p)) => format!("{}:{}@", encode_component(u), encode_component(p)),
                (Some(u), None) => format!("{}@", encode_component(u)),
                _ => String::new(),
            };
            Some(format!("socks5h://{}{}", auth, address))
        }
        UpstreamType::Socks4 { address, user_id, .. } => {
            let auth = user_id
                .as_deref()
                .map(|u| format!("{}@", encode_component(u)))
                .unwrap_or_default();
            Some(format!("socks4://{}{}", auth, address))
        }
        UpstreamType::Direct { .. } => None,
    }
}

pub fn select_socks_proxy_url(upstreams: &[UpstreamConfig]) -> Option<String> {
    let unscoped = upstreams
        .iter()
        .filter(|u| u.enabled && u.scopes.trim().is_empty())
        .find_map(upstream_to_socks_url);
    if unscoped.is_some() {
        return unscoped;
    }

    upstreams
        .iter()
        .filter(|u| u.enabled)
        .find_map(upstream_to_socks_url)
}

pub fn build_http_client(socks_proxy_url: Option<&str>) -> Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder();
    if let Some(proxy_url) = socks_proxy_url {
        let proxy = reqwest::Proxy::all(proxy_url)
            .map_err(|e| ProxyError::Proxy(format!("Invalid SOCKS proxy URL '{proxy_url}': {e}")))?;
        builder = builder.proxy(proxy);
    }
    builder
        .build()
        .map_err(|e| ProxyError::Proxy(format!("Failed to build HTTP client: {e}")))
}
