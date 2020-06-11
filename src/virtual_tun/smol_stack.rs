//use smoltcp_openvpn_bridge::virtual_tun::VirtualTunInterface;
use super::interface::{CIpv4Address, CIpv4Cidr, CIpv6Address, CIpv6Cidr};
use super::virtual_tun::VirtualTunInterface as TunDevice;
use smoltcp::iface::{NeighborCache, Interface, InterfaceBuilder};
use smoltcp::phy::{self, Device};
use smoltcp::wire::{IpEndpoint, IpVersion, IpProtocol};
use smoltcp::storage::{PacketMetadata};
use smoltcp::socket::{Socket, SocketSet, TcpSocket, TcpSocketBuffer, UdpSocket, UdpSocketBuffer, RawSocket, RawSocketBuffer};
use std::collections::BTreeMap;

pub struct TunSmolStack<'a, 'b: 'a, 'c: 'a + 'b> {
    sockets: SocketSet<'a, 'b, 'c >,
    interface: Interface<'a, 'a, 'a, TunDevice>
}

pub enum SocketType {
    RAW_IPV4,
    RAW_IPV6,
    TCP,
    UDP,
}

//TODO: why I cant do TunSmolStack<'a, 'b, 'c, 'e, DeviceT: for<'d> Device<'d>>?
impl<'a, 'b: 'a, 'c: 'a + 'b> TunSmolStack<'a, 'b, 'c> {
    pub fn new(interface_name: String) -> Result<TunSmolStack<'a, 'b, 'c>, u32> {
        let device = TunDevice::new(interface_name.as_str()).unwrap();
        let neighbor_cache = NeighborCache::new(BTreeMap::new());
        let socket_set = SocketSet::new(vec![]);
        let mut interface = InterfaceBuilder::new(device)
            .neighbor_cache(neighbor_cache)
            .finalize();
        Ok(TunSmolStack {
            sockets: socket_set,
            interface: interface,
        })
    }

    pub fn add_socket(&mut self, socket_type: SocketType) -> usize {
        match socket_type {
            SocketType::TCP => {
                let rx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
                let tx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
                let socket = TcpSocket::new(rx_buffer, tx_buffer);
                let handle = self.sockets.add(socket); 
                //self.sockets.add(Socket::Tcp(socket));      
            }
            
            SocketType::UDP => {
                let rx_buffer = UdpSocketBuffer::new(Vec::new(), vec![0; 1024]);
                let tx_buffer = UdpSocketBuffer::new(Vec::new(), vec![0; 1024]);
                let socket = UdpSocket::new(rx_buffer, tx_buffer);
                let handle = self.sockets.add(socket);
            }
            /*
            SocketType::RAW_IPV4 => {
                let rx_buffer = RawSocketBuffer::new(Vec::new(), vec![0; 1024]);
                let tx_buffer = RawSocketBuffer::new(Vec::new(), vec![0; 1024]);
                //TODO: which protocol?
                let socket = RawSocket::new(IpVersion::Ipv4,IpProtocol::Tcp,rx_buffer, tx_buffer);
                self.sockets.add(socket);
            }

            SocketType::RAW_IPV6 => {
                let rx_buffer = RawSocketBuffer::new(Vec::new(), vec![0; 1024]);
                let tx_buffer = RawSocketBuffer::new(Vec::new(), vec![0; 1024]);
                //TODO: which protocol?
                let socket = RawSocket::new(IpVersion::Ipv4,IpProtocol::Tcp,rx_buffer, tx_buffer);
                self.sockets.add(socket);
            }
            */
            _ => {
                panic!{"wrong choice for socket type"}
            }
        }
        0
    }
    /*
    pub fn add_ipv4_address(&mut self, cidr: CIpv4Cidr) -> Self {
        *self
    }

    pub fn add_ipv6_address(&mut self, cidr: CIpv6Cidr) -> Self {
        *self
    }

    pub fn add_default_v4_gateway(&mut self, ipv4_address: CIpv4Address) -> Self {
        *self
    }

    pub fn add_default_v6_gateway(&mut self, ipv4_address: CIpv6Address) -> Self {
        *self
    }
    */
}
