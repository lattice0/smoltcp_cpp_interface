//use smoltcp_openvpn_bridge::virtual_tun::VirtualTunInterface;
use super::interface::{CIpv4Address, CIpv4Cidr, CIpv6Address, CIpv6Cidr};
use super::virtual_tun::VirtualTunInterface as TunDevice;
use smoltcp::iface::{NeighborCache, Interface, InterfaceBuilder};
use smoltcp::phy::{self, Device};
use smoltcp::socket::{SocketSet, TcpSocket, TcpSocketBuffer, UdpSocket, UdpSocketBuffer, RawSocket, RawSocketBuffer};
use std::collections::BTreeMap;

pub struct TunSmolStack<'a, 'b, 'c, 'd, 'e, 'f> {
    device: TunDevice,
    sockets: SocketSet<'a, 'b, 'c >,
    interface: Interface<'d, 'e, 'f, TunDevice>
}

pub enum SocketType {
    RAW,
    TCP,
    UDP,
}

//TODO: why I cant do TunSmolStack<'a, 'b, 'c, 'e, DeviceT: for<'d> Device<'d>>?
impl<'a, 'b, 'c, 'd, 'e, 'f> TunSmolStack<'a, 'b, 'c, 'd, 'e, 'f> {
    pub fn new(interface_name: String) -> Result<TunSmolStack<'a, 'b, 'c, 'd, 'e, 'f>, u32> {
        let device = TunDevice::new("tun").unwrap();
        //let rx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
        //let tx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
    
        let neighbor_cache = NeighborCache::new(BTreeMap::new());
        let socket_set = SocketSet::new(vec![]);
        let mut interface = InterfaceBuilder::new(device)
            .neighbor_cache(neighbor_cache)
            .finalize();
        Ok(TunSmolStack {
            device: device,
            sockets: socket_set,
            interface: interface,
        })
    }

    pub fn add_socket(mut self, socket_type: SocketType) -> usize {
        match socket_type {
            SocketType::TCP => {
                let rx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
                let tx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
                let socket = TcpSocket::new(rx_buffer, tx_buffer);
                self.sockets.add(socket);
            }
            /*
            SocketType::UDP => {
                let rx_buffer = UdpSocketBuffer::new(vec![0; 1024]);
                let tx_buffer = UdpSocketBuffer::new(vec![0; 1024]);
                let socket = UdpSocket::new(rx_buffer, tx_buffer);
                self.sockets.add(socket);
            }
            */
            /*
            SocketType::RAW => {
                let rx_buffer = RawSocketBuffer::new(vec![0; 1024]);
                let tx_buffer = RawSocketBuffer::new(vec![0; 1024]);
                let socket = RawSocket::new(rx_buffer, tx_buffer);
                self.sockets.add(socket);
            }
            */
        }
        0
    }

    pub fn add_ipv4_address(mut self, cidr: CIpv4Cidr) -> Self {
        self
    }

    pub fn add_ipv6_address(mut self, cidr: CIpv6Cidr) -> Self {
        self
    }

    pub fn add_default_v4_gateway(mut self, ipv4_address: CIpv4Address) -> Self {
        self
    }

    pub fn add_default_v6_gateway(mut self, ipv4_address: CIpv6Address) -> Self {
        self
    }
}
