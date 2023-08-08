use std::io;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::time::Duration;

use super::config::DEFAULT_DISCOVERY_PORT;
use super::mobile_contract::{DiscoveryAnnouncement, ServiceIdentity};

pub const DISCOVERY_PROBE_V1: &str = "GSCALE_DISCOVER_V1";
pub const DISCOVERY_ANNOUNCE_INTERVAL_MS: u64 = 250;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoverySocketConfig {
    pub bind_addr: SocketAddrV4,
    pub announce_targets: Vec<SocketAddrV4>,
    pub read_timeout: Duration,
}

impl DiscoverySocketConfig {
    pub fn new(bind_ip: Ipv4Addr, discovery_port: u16, announce_targets: Vec<Ipv4Addr>) -> Self {
        let port = if discovery_port == 0 {
            DEFAULT_DISCOVERY_PORT
        } else {
            discovery_port
        };
        let announce_targets = normalize_announce_targets(announce_targets, port);

        Self {
            bind_addr: SocketAddrV4::new(bind_ip, port),
            announce_targets,
            read_timeout: Duration::from_millis(250),
        }
    }
}

pub fn discovery_response_for_packet(
    packet: &[u8],
    identity: &ServiceIdentity,
    http_port: u16,
    candidate_ports: Vec<u16>,
) -> Option<Vec<u8>> {
    if !is_discovery_probe(packet) {
        return None;
    }
    DiscoveryAnnouncement::new(identity, http_port, candidate_ports)
        .to_json_bytes()
        .ok()
}

pub fn is_discovery_probe(packet: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(packet) else {
        return false;
    };
    text.trim() == DISCOVERY_PROBE_V1
}

pub fn bind_discovery_socket(config: &DiscoverySocketConfig) -> io::Result<UdpSocket> {
    let socket = UdpSocket::bind(config.bind_addr)?;
    socket.set_broadcast(true)?;
    socket.set_read_timeout(Some(config.read_timeout))?;
    Ok(socket)
}

pub fn send_discovery_announcement(
    socket: &UdpSocket,
    config: &DiscoverySocketConfig,
    identity: &ServiceIdentity,
    http_port: u16,
    candidate_ports: Vec<u16>,
) -> io::Result<usize> {
    let payload = DiscoveryAnnouncement::new(identity, http_port, candidate_ports)
        .to_json_bytes()
        .map_err(io::Error::other)?;
    let mut sent = 0;
    for target in &config.announce_targets {
        socket.send_to(&payload, target)?;
        sent += 1;
    }
    Ok(sent)
}

pub fn broadcast_targets_from_ipv4_networks(
    networks: &[(Ipv4Addr, Ipv4Addr)],
    discovery_port: u16,
) -> Vec<SocketAddrV4> {
    let port = if discovery_port == 0 {
        DEFAULT_DISCOVERY_PORT
    } else {
        discovery_port
    };
    let mut targets = vec![SocketAddrV4::new(Ipv4Addr::new(255, 255, 255, 255), port)];

    for (ip, mask) in networks {
        if !is_private_ipv4(*ip) {
            continue;
        }
        let broadcast = ipv4_broadcast(*ip, *mask);
        let target = SocketAddrV4::new(broadcast, port);
        if !targets.contains(&target) {
            targets.push(target);
        }
    }

    targets
}

fn normalize_announce_targets(
    announce_targets: Vec<Ipv4Addr>,
    discovery_port: u16,
) -> Vec<SocketAddrV4> {
    if announce_targets.is_empty() {
        return vec![SocketAddrV4::new(
            Ipv4Addr::new(255, 255, 255, 255),
            discovery_port,
        )];
    }

    let mut out = Vec::new();
    for target in announce_targets {
        let socket = SocketAddrV4::new(target, discovery_port);
        if !out.contains(&socket) {
            out.push(socket);
        }
    }
    out
}

fn ipv4_broadcast(ip: Ipv4Addr, mask: Ipv4Addr) -> Ipv4Addr {
    let ip = u32::from(ip);
    let mask = u32::from(mask);
    Ipv4Addr::from(ip | !mask)
}

fn is_private_ipv4(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();
    octets[0] == 10
        || (octets[0] == 172 && (16..=31).contains(&octets[1]))
        || (octets[0] == 192 && octets[1] == 168)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn identity() -> ServiceIdentity {
        ServiceIdentity::new("rp-scale", "dev-operator", "Operator One", "admin")
    }

    #[test]
    fn recognizes_exact_gscale_probe_with_whitespace() {
        assert!(is_discovery_probe(b"GSCALE_DISCOVER_V1"));
        assert!(is_discovery_probe(b" GSCALE_DISCOVER_V1\n"));
        assert!(!is_discovery_probe(b"GSCALE_DISCOVER_V2"));
        assert!(!is_discovery_probe(&[0xff, 0xfe]));
    }

    #[test]
    fn returns_announcement_only_for_probe_packet() {
        let response =
            discovery_response_for_packet(b"GSCALE_DISCOVER_V1", &identity(), 39117, vec![39117])
                .unwrap();
        let decoded: Value = serde_json::from_slice(&response).unwrap();

        assert_eq!(decoded["type"], "gscale_announce_v1");
        assert_eq!(decoded["service"], "mobileapi");
        assert_eq!(decoded["server_name"], "rp-scale");
        assert_eq!(decoded["http_port"], 39117);
        assert!(discovery_response_for_packet(b"unknown", &identity(), 39117, vec![]).is_none());
    }

    #[test]
    fn computes_broadcast_targets_like_gscale_mobileapi() {
        let targets = broadcast_targets_from_ipv4_networks(
            &[
                (
                    Ipv4Addr::new(192, 168, 1, 10),
                    Ipv4Addr::new(255, 255, 255, 0),
                ),
                (Ipv4Addr::new(10, 42, 0, 80), Ipv4Addr::new(255, 255, 0, 0)),
                (Ipv4Addr::new(8, 8, 8, 8), Ipv4Addr::new(255, 255, 255, 0)),
            ],
            18081,
        );

        assert!(targets.contains(&SocketAddrV4::new(Ipv4Addr::new(255, 255, 255, 255), 18081)));
        assert!(targets.contains(&SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 255), 18081)));
        assert!(targets.contains(&SocketAddrV4::new(Ipv4Addr::new(10, 42, 255, 255), 18081)));
        assert_eq!(targets.len(), 3);
    }

    #[test]
    fn socket_config_defaults_to_gscale_discovery_port_and_broadcast() {
        let config = DiscoverySocketConfig::new(Ipv4Addr::UNSPECIFIED, 0, vec![]);

        assert_eq!(
            config.bind_addr,
            SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 18081)
        );
        assert_eq!(
            config.announce_targets,
            vec![SocketAddrV4::new(Ipv4Addr::new(255, 255, 255, 255), 18081)]
        );
    }
}
