use std::net::SocketAddr;
use std::net::{IpAddr, Ipv4Addr};
use async_channel::TryRecvError;
use tokio::net::UdpSocket;
use tokio::task::JoinHandle;
use crate::error::Error;

use async_channel::Sender;
use async_channel::Receiver;
use async_channel::unbounded;

fn package(identifier: &'static str, payload: [u8; 4]) -> Vec<u8> {
    let mut bytes = vec![];
    bytes.append(&mut identifier.as_bytes().to_vec());
    bytes.append(&mut payload.to_vec());
    bytes
}

pub enum ThreadMessage {
    Kill
}

pub struct Server {
    killswitch: Sender<ThreadMessage>,
    thread: JoinHandle<Result<(), Error>>
}

impl Server {

    pub async fn spawn(identifier: &'static str, port: u16) -> Self {
        let (killswitch_sender, killswitch_receiver) = unbounded();
        let thread = tokio::spawn(Self::run(identifier, killswitch_receiver, port));

        Self {
            killswitch: killswitch_sender,
            thread
        }
    }

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
    pub async fn run(identifier: &'static str, killswitch: Receiver<ThreadMessage>, port: u16) -> Result<(), Error> {
        let ip_addr = Self::find_suitable_ipv4().await?;
        let socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port)).await.map_err(|_| Error::BindFailed)?;
        socket.set_broadcast(true).map_err(|_| Error::BroadcastFailed)?;
        
        let bytes = package(identifier, ip_addr.octets());
        let mut buf = vec![0u8; identifier.len()];

        while let Err(e) = killswitch.try_recv() {
            if e == TryRecvError::Closed { break; }
            let (_, addr) = socket.recv_from(&mut buf).await.map_err(|_| Error::RecvFailed)?;

            // If the parsed string matches the identifer, echo back the server address
            if String::from_utf8_lossy(&bytes).to_string().as_str() == identifier {
                socket.send_to(&bytes, addr).await.map_err(|_| Error::BroadcastFailed)?;
            }
        }

        Ok(())
    }

    /// Stop UDP broadcasts
    pub fn stop(&self) {
        let _ = self.killswitch.send_blocking(ThreadMessage::Kill);
    }

    pub async fn wait(self) {
        let _ = self.thread.await;
    }
}
