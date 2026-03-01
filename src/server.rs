use std::net::{IpAddr, Ipv4Addr};
use tokio::net::UdpSocket;
use crate::error::Error;

pub struct Server {

}

impl Server {

    /// Find a suitable ipv4 address for LAN connection
    pub async fn find_suitable_ipv4() -> Result<Ipv4Addr, Error> {
        for iface in if_addrs::get_if_addrs().map_err(|_| Error::NoEndpoint)? {
            if !iface.is_loopback() && let IpAddr::V4(ipv4addr) = iface.ip() {
                return Ok(ipv4addr)
            }
        }

        Err(Error::NoEndpoint)
    }

    /// Bind a UDP socket to all interfaces to transmit IP address
    pub async fn bind() -> Result<(), Error> {
        let ip_addr = Self::find_suitable_ipv4().await?;
        let socket = UdpSocket::bind("0.0.0.0:34254").await.map_err(|_| Error::BindFailed)?;
        Ok(())
    }
}
