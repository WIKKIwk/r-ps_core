use std::net::{TcpListener, ToSocketAddrs};

pub const DEFAULT_DISCOVERY_PORT: u16 = 18081;
pub const DEFAULT_MOBILE_API_PORTS: &[u16] = &[39117, 41257, 43391, 45533, 47681];

#[derive(Clone, Debug, PartialEq)]
pub struct MobileServiceConfig {
    pub listen_host: String,
    pub listen_addr: String,
    pub discovery_addr: String,
    pub candidate_ports: Vec<u16>,
    pub server_name: String,
}

impl MobileServiceConfig {
    pub fn new(
        listen_host: &str,
        explicit_listen_addr: &str,
        candidate_ports: Vec<u16>,
        server_name: &str,
    ) -> Self {
        let candidate_ports = normalize_candidate_ports(candidate_ports);
        let listen_host = normalize_listen_host(listen_host);
        let listen_addr = select_listen_addr(explicit_listen_addr, &listen_host, &candidate_ports);

        Self {
            listen_host,
            listen_addr,
            discovery_addr: format!("0.0.0.0:{DEFAULT_DISCOVERY_PORT}"),
            candidate_ports,
            server_name: normalize_server_name(server_name),
        }
    }

    pub fn http_port(&self) -> u16 {
        port_from_listen_addr(&self.listen_addr).unwrap_or_else(default_mobile_api_port)
    }
}

pub fn default_mobile_api_port() -> u16 {
    DEFAULT_MOBILE_API_PORTS[0]
}

pub fn parse_candidate_ports(raw: &str) -> Vec<u16> {
    let mut out = Vec::new();
    for part in raw.split(',') {
        let Ok(port) = part.trim().parse::<u16>() else {
            continue;
        };
        if port == 0 || out.contains(&port) {
            continue;
        }
        out.push(port);
    }
    normalize_candidate_ports(out)
}

pub fn select_listen_addr(explicit_addr: &str, bind_host: &str, candidate_ports: &[u16]) -> String {
    let explicit_addr = explicit_addr.trim();
    if !explicit_addr.is_empty() {
        return explicit_addr.to_string();
    }

    let bind_host = normalize_listen_host(bind_host);
    let candidates = normalize_candidate_ports(candidate_ports.to_vec());
    for port in &candidates {
        let addr = format!("{bind_host}:{port}");
        if is_tcp_listen_addr_available(&addr) {
            return addr;
        }
    }

    format!("{}:{}", bind_host, candidates[0])
}

pub fn port_from_listen_addr(addr: &str) -> Option<u16> {
    let addr = addr.trim();
    if addr.is_empty() {
        return None;
    }
    if let Some(port) = addr.strip_prefix(':') {
        return port.parse::<u16>().ok().filter(|port| *port > 0);
    }
    addr.rsplit_once(':')
        .and_then(|(_, port)| port.parse::<u16>().ok())
        .filter(|port| *port > 0)
}

fn normalize_candidate_ports(mut candidate_ports: Vec<u16>) -> Vec<u16> {
    if candidate_ports.is_empty() {
        return DEFAULT_MOBILE_API_PORTS.to_vec();
    }
    candidate_ports.retain(|port| *port > 0);
    candidate_ports.dedup();
    if candidate_ports.is_empty() {
        DEFAULT_MOBILE_API_PORTS.to_vec()
    } else {
        candidate_ports
    }
}

fn normalize_listen_host(raw: &str) -> String {
    match raw.trim() {
        "" => "0.0.0.0".to_string(),
        value => value.to_string(),
    }
}

fn normalize_server_name(raw: &str) -> String {
    match raw.trim() {
        "" => "gscale-zebra".to_string(),
        value => value.to_string(),
    }
}

fn is_tcp_listen_addr_available(addr: &str) -> bool {
    let Ok(mut addrs) = addr.to_socket_addrs() else {
        return false;
    };
    addrs.any(|addr| TcpListener::bind(addr).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn parses_candidate_ports_like_gscale_mobileapi() {
        assert_eq!(
            parse_candidate_ports("39117, 41257, bad, 41257, 0"),
            vec![39117, 41257]
        );
        assert_eq!(parse_candidate_ports("bad"), DEFAULT_MOBILE_API_PORTS);
    }

    #[test]
    fn explicit_listen_addr_wins() {
        assert_eq!(
            select_listen_addr("0.0.0.0:8081", "127.0.0.1", &[39117]),
            "0.0.0.0:8081"
        );
    }

    #[test]
    fn chooses_first_free_candidate_port() {
        let busy = TcpListener::bind("127.0.0.1:0").unwrap();
        let busy_port = busy.local_addr().unwrap().port();
        let free = TcpListener::bind("127.0.0.1:0").unwrap();
        let free_port = free.local_addr().unwrap().port();
        drop(free);

        assert_eq!(
            select_listen_addr("", "127.0.0.1", &[busy_port, free_port]),
            format!("127.0.0.1:{free_port}")
        );
    }

    #[test]
    fn config_exposes_http_and_discovery_ports() {
        let cfg = MobileServiceConfig::new("127.0.0.1", "127.0.0.1:41257", vec![], "");

        assert_eq!(cfg.http_port(), 41257);
        assert_eq!(cfg.discovery_addr, "0.0.0.0:18081");
        assert_eq!(cfg.server_name, "gscale-zebra");
        assert_eq!(cfg.candidate_ports, DEFAULT_MOBILE_API_PORTS);
    }
}
