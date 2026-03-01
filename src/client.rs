use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::UdpSocket;
use crate::error::Error;

pub async fn discover(identifier: &'static str, port: u16) -> Result<IpAddr, Error> {
    let socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port)).await.map_err(|e| {println!("ERROR: {e:?}"); Error::BindFailed})?;
    socket.set_broadcast(true).map_err(|_| Error::BroadcastFailed)?;

    let buffer = identifier.as_bytes().to_vec();
    socket.send_to(&buffer, SocketAddr::new(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)), port)).await.map_err(|_| Error::BroadcastFailed)?;
    let mut recv_buffer = vec![0u8; identifier.len() + 4];
    let (bytes_received, _) = socket.recv_from(&mut recv_buffer).await.map_err(|_| Error::RecvFailed)?;

    println!("Bytes received: {bytes_received}/{}", recv_buffer.len());
    println!("Buffer: {recv_buffer:?}");

    if bytes_received != recv_buffer.len() {
        return Err(Error::InvalidIdentifier);
    }

    let parsed_identifier = String::from_utf8_lossy(&recv_buffer[0..identifier.len()]).to_string();

    if parsed_identifier != identifier {
        return Err(Error::InvalidIdentifier);
    }

    let segments = &recv_buffer[identifier.len() + 1..];
    Ok(IpAddr::V4(Ipv4Addr::from_octets(segments.try_into().map_err(|_| Error::InvalidIdentifier)?)))
}
