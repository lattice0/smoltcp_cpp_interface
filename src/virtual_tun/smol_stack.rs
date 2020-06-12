//use smoltcp_openvpn_bridge::virtual_tun::VirtualTunInterface;
use super::interface::{CIpv4Address, CIpv4Cidr, CIpv6Address, CIpv6Cidr};
use super::virtual_tun::VirtualTunInterface as TunDevice;
use smoltcp::iface::{NeighborCache, Interface, InterfaceBuilder, Routes, NeighborCache};
use smoltcp::phy::{self, Device};
use smoltcp::wire::{IpEndpoint, IpVersion, IpProtocol, IpCidr, Ipv4Address, Ipv6Address};
use smoltcp::storage::{PacketMetadata};
use smoltcp::socket::{Socket, SocketSet, SocketHandle, TcpSocket, TcpSocketBuffer, UdpSocket, UdpSocketBuffer, RawSocket, RawSocketBuffer};
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

pub struct TunSmolStackBuilder<'a, 'b: 'a, 'c: 'a + 'b> {
    sockets: SocketSet<'a, 'b, 'c >,
    device: TunDevice,
    ip_addrs: std::vec::Vec<IpCidr>,
    default_v4_gw: Option<Ipv4Address>,
    default_v6_gw: Option<Ipv6Address>,
    neighbor_cache: Option<NeighborCache<'a>>,
    //interface: Interface<'a, 'a, 'a, TunDevice>
}

//TODO: why I cant do TunSmolStack<'a, 'b, 'c, 'e, DeviceT: for<'d> Device<'d>>?
impl<'a, 'b: 'a, 'c: 'a + 'b> TunSmolStackBuilder<'a, 'b, 'c> {
    pub fn new(interface_name: String) -> TunSmolStackBuilder<'a, 'b, 'c> {
        let socket_set = SocketSet::new(vec![]);
        let neighbor_cache = NeighborCache::new(BTreeMap::new());
        let device = TunDevice::new(interface_name.as_str()).unwrap();
        TunSmolStackBuilder {
            sockets: socket_set,
            device: device,
            ip_addrs: std::vec::Vec::new(),
            default_v4_gw: None,
            default_v6_gw: None,
            neighbor_cache: Some(neighbor_cache),
        }
    }

    pub fn add_socket(&mut self, socket_type: SocketType) -> usize {
        match socket_type {
            SocketType::TCP => {
                let rx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
                let tx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
                let socket = TcpSocket::new(rx_buffer, tx_buffer);
                let handle = self.sockets.add(socket); 
                handle.value()
            }
            
            SocketType::UDP => {
                let rx_buffer = UdpSocketBuffer::new(Vec::new(), vec![0; 1024]);
                let tx_buffer = UdpSocketBuffer::new(Vec::new(), vec![0; 1024]);
                let socket = UdpSocket::new(rx_buffer, tx_buffer);
                let handle = self.sockets.add(socket);
                handle.value()
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
    }

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

    pub fn finalize(&mut self) -> TunSmolStack {
        let mut routes_storage = [None; 2];
        let mut routes = Routes::new(&mut routes_storage[..]);
        routes.add_default_ipv4_route(self.default_v4_gw.unwrap()).unwrap();
        routes.add_default_ipv6_route(self.default_v6_gw.unwrap()).unwrap();
        
        let mut interface = InterfaceBuilder::new(self.device)
            .neighbor_cache(self.neighbor_cache.unwrap())
            .ip_addrs(self.ip_addrs)
            .routes(routes)
            .finalize();
            
        TunSmolStack {
            sockets: self.sockets,
            interface: interface
        }
    }
    
}
