//! Implements <https://datatracker.ietf.org/doc/rfc1035/>

pub mod message;

use std::{
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, UdpSocket},
};

use crate::message::Message;

const MAX_DATAGRAM_SIZE: usize = 1024;
const UDP_SOCKET: &'static str = "0.0.0.0:20000";
const NAMESERVER: &'static str = "8.8.8.8:53";

#[derive(Debug)]
pub enum DNSError {
    FailedToBindSocket,
    ConnectionRefused,
    InvalidResponse,
    NetworkError,
}

pub fn resolve(domain_name: &[u8]) -> Result<IpAddr, DNSError> {
    // Bind a UDP socket
    let socket = UdpSocket::bind(UDP_SOCKET).map_err(|_| DNSError::FailedToBindSocket)?;
    socket.connect(NAMESERVER).unwrap(); // .map_err(|_| DNSError::ConnectionRefused)?;

    // Send a DNS query
    let message = Message::new(domain_name);
    let mut bytes = vec![0; message.size()];
    message.write_to_buffer(&mut bytes);
    socket.send(&bytes).map_err(|_| DNSError::NetworkError)?;

    // Read the DNS response
    let mut response = [0; MAX_DATAGRAM_SIZE];
    let response_length = socket.recv(&mut response).map_err(|_| DNSError::NetworkError)?;
    Message::read(&response[..response_length]).map_err(|_| DNSError::InvalidResponse)?;
    todo!();
}
