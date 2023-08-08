use serde::{Deserialize, Serialize};

use super::config::{DEFAULT_DISCOVERY_PORT, DEFAULT_MOBILE_API_PORTS, default_mobile_api_port};

pub const APP_ID: &str = "gscale-zebra";
pub const SERVICE_ID: &str = "mobileapi";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceIdentity {
    pub server_name: String,
    pub server_ref: String,
    pub display_name: String,
    pub role: String,
}

impl ServiceIdentity {
    pub fn new(server_name: &str, server_ref: &str, display_name: &str, role: &str) -> Self {
        Self {
            server_name: normalize(server_name, APP_ID),
            server_ref: normalize(server_ref, "unknown"),
            display_name: normalize(display_name, "Operator"),
            role: normalize(role, "operator"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct HealthResponse {
    pub ok: bool,
    pub service: &'static str,
}

impl HealthResponse {
    pub fn ok() -> Self {
        Self {
            ok: true,
            service: SERVICE_ID,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct HandshakeResponse {
    pub ok: bool,
    pub service: &'static str,
    pub app: &'static str,
    pub server_name: String,
    pub server_ref: String,
    pub display_name: String,
    pub role: String,
    pub phone: String,
    pub http_port: u16,
    pub discovery_port: u16,
    pub candidate_ports: Vec<u16>,
    pub monitor_path: &'static str,
    pub profile_path: &'static str,
    pub items_path: &'static str,
    pub batch_state_path: &'static str,
    pub requires_auth: bool,
}

impl HandshakeResponse {
    pub fn new(identity: &ServiceIdentity, http_port: u16, candidate_ports: Vec<u16>) -> Self {
        Self {
            ok: true,
            service: SERVICE_ID,
            app: APP_ID,
            server_name: identity.server_name.clone(),
            server_ref: identity.server_ref.clone(),
            display_name: identity.display_name.clone(),
            role: identity.role.clone(),
            phone: String::new(),
            http_port: normalize_port(http_port),
            discovery_port: DEFAULT_DISCOVERY_PORT,
            candidate_ports: normalize_candidate_ports(candidate_ports),
            monitor_path: "/v1/mobile/monitor/state",
            profile_path: "/v1/mobile/profile",
            items_path: "/v1/mobile/items",
            batch_state_path: "/v1/mobile/batch/state",
            requires_auth: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DiscoveryAnnouncement {
    #[serde(rename = "type")]
    pub announcement_type: &'static str,
    pub app: &'static str,
    pub service: &'static str,
    pub server_name: String,
    pub server_ref: String,
    pub display_name: String,
    pub role: String,
    pub http_port: u16,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub candidate_ports: Vec<u16>,
}

impl DiscoveryAnnouncement {
    pub fn new(identity: &ServiceIdentity, http_port: u16, candidate_ports: Vec<u16>) -> Self {
        Self {
            announcement_type: "gscale_announce_v1",
            app: APP_ID,
            service: SERVICE_ID,
            server_name: identity.server_name.clone(),
            server_ref: identity.server_ref.clone(),
            display_name: identity.display_name.clone(),
            role: identity.role.clone(),
            http_port: normalize_port(http_port),
            candidate_ports: normalize_candidate_ports(candidate_ports),
        }
    }

    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}

fn normalize(value: &str, fallback: &str) -> String {
    match value.trim() {
        "" => fallback.to_string(),
        value => value.replace(['\n', '\r'], " "),
    }
}

fn normalize_port(port: u16) -> u16 {
    if port == 0 {
        default_mobile_api_port()
    } else {
        port
    }
}

fn normalize_candidate_ports(candidate_ports: Vec<u16>) -> Vec<u16> {
    if candidate_ports.is_empty() {
        DEFAULT_MOBILE_API_PORTS.to_vec()
    } else {
        candidate_ports
            .into_iter()
            .filter(|port| *port > 0)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> ServiceIdentity {
        ServiceIdentity::new("rp-scale", "dev-operator", "Operator One", "admin")
    }

    #[test]
    fn builds_gscale_compatible_handshake_shape() {
        let handshake = HandshakeResponse::new(&identity(), 39117, vec![39117, 41257]);

        assert!(handshake.ok);
        assert_eq!(handshake.service, "mobileapi");
        assert_eq!(handshake.app, "gscale-zebra");
        assert_eq!(handshake.server_name, "rp-scale");
        assert_eq!(handshake.discovery_port, 18081);
        assert_eq!(handshake.monitor_path, "/v1/mobile/monitor/state");
        assert!(!handshake.requires_auth);
    }

    #[test]
    fn builds_gscale_compatible_discovery_announcement_json() {
        let payload = DiscoveryAnnouncement::new(&identity(), 39117, vec![39117, 41257]);
        let json = String::from_utf8(payload.to_json_bytes().unwrap()).unwrap();

        assert!(json.contains(r#""type":"gscale_announce_v1""#));
        assert!(json.contains(r#""service":"mobileapi""#));
        assert!(json.contains(r#""app":"gscale-zebra""#));
        assert!(json.contains(r#""http_port":39117"#));
        assert!(json.contains(r#""candidate_ports":[39117,41257]"#));
    }

    #[test]
    fn health_response_matches_mobile_fallback_probe() {
        assert_eq!(HealthResponse::ok().service, "mobileapi");
    }
}
