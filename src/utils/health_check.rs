use std::env;
use std::net::{TcpStream, UdpSocket};
use std::time::Duration;

#[derive(Debug)]
pub struct HealthCheckConfig {
    pub host: String,
    pub port: u16,
    pub protocol: String,
}

pub fn get_service_config(service_name: &str) -> Option<HealthCheckConfig> {
    let upper_service: String = service_name.to_uppercase();
    let host_key: String = format!("{}_HEALTH_HOST", upper_service); // Hostname
    let port_key: String = format!("{}_HEALTH_PORT", upper_service); // Port Number
    let proc_key: String = format!("{}_HEALTH_TYPE", upper_service); // Protocal

    let host: String = env::var(&host_key).ok()?;
    let port: u16 = env::var(&port_key)
        .ok()?
        .parse::<u16>()
        .ok()?;
    let protocol: String = env::var(&proc_key).ok()?;

    if protocol != "TCP" && protocol != "UDP" {
        return None;
    }

    Some(HealthCheckConfig {
        host,
        port,
        protocol,
    })
}

pub fn resolve_host(host: &str) -> Option<String> {
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        return Some(ip.to_string());
    }

    use std::net::ToSocketAddrs;
    match (host, 0).to_socket_addrs() {
        Ok(mut ips) => ips.next().map(|socket_addr| socket_addr.ip().to_string()),
        Err(_) => None,
    }
}

pub fn check_service_health(config: &HealthCheckConfig) -> bool {
    let host: Option<String> = resolve_host(&config.host);
    if host.is_none() {
        return false;
    };

    match config.protocol.as_str() {
        "TCP" => check_tcp_health(host.unwrap().as_str(), config.port),
        "UDP" => check_udp_health(host.unwrap().as_str(), config.port),
        _ => false,
    }
}

fn check_tcp_health(host: &str, port: u16) -> bool {
    let addr: String = format!("{}:{}", host, port);
    match addr.parse() {
        Ok(socket_addr) => match TcpStream::connect_timeout(&socket_addr, Duration::from_secs(5)) {
            Ok(_) => true,
            Err(_) => false,
        },
        Err(_) => false,
    }
}

fn check_udp_health(host: &str, port: u16) -> bool {
    let addr: String = format!("{}:{}", host, port);
    match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => {
            if let Ok(_) = socket.connect(&addr) {
                match socket.send(&[1]) {
                    Ok(_) => {
                        let mut buf = [0; 1];
                        if let Ok(_) = socket.set_read_timeout(Some(Duration::from_secs(5))) {
                            match socket.recv(&mut buf) {
                                Ok(_) => true,
                                Err(_) => false,
                            }
                        } else {
                            false
                        }
                    },
                    Err(_) => false,
                }
            } else {
                false
            }
        },
        Err(_) => false,
    }
}
