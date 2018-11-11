use lazy_static::*;
use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs};
use std::io::Result;
use std::sync::Arc;

lazy_static! {
    static ref BUFFERS: HashMap<SocketAddr, Arc<Vec<u8>>> = HashMap::new();
}

#[derive(Debug)]
pub struct UdpSocket {
    addr: SocketAddr,
}

impl UdpSocket {
    
    pub fn bind<A: ToSocketAddrs>(addrs: A) -> Result<UdpSocket> {
        let addr = addrs.to_socket_addrs()?.next().unwrap();
        Ok(UdpSocket { addr })
    }



}