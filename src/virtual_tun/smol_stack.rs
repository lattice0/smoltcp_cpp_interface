//use smoltcp_openvpn_bridge::virtual_tun::VirtualTunInterface;
use super::interface::{CIpv4Address, CIpv4Cidr, CIpv6Address, CIpv6Cidr};
use super::virtual_tun::VirtualTunInterface as TunDevice;
use smoltcp::iface::{Interface, InterfaceBuilder, Routes};
use smoltcp::phy::{self, Device};
use smoltcp::socket::{
    RawSocket, RawSocketBuffer, Socket, SocketHandle, SocketSet, TcpSocket, TcpSocketBuffer,
    UdpSocket, UdpSocketBuffer,
};
use smoltcp::storage::PacketMetadata;
use smoltcp::wire::{
    IpAddress, IpCidr, IpEndpoint, IpProtocol, IpVersion, Ipv4Address, Ipv6Address,
};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::VecDeque;

pub enum SocketType {
    RAW_IPV4,
    RAW_IPV6,
    TCP,
    UDP,
}

pub struct Blob {
    data: *mut u8, 
    len: usize
}

pub struct SmolSocket<'a, 'b: 'a, 'c: 'a + 'b> {
    pub tun_smol_stack: Rc<RefCell<TunSmolStack<'a, 'b, 'c>>>,
    pub packets: VecDeque<Blob>
}

impl<'a, 'b: 'a, 'c: 'a + 'b> SmolSocket<'a, 'b, 'c> {
    pub fn new(interface_name: String, tun_smol_stack: Rc<RefCell<TunSmolStack<'a, 'b, 'c>>>) -> Rc<RefCell<SmolSocket<'a, 'b, 'c>> {
        SmolSocket {
            tun_smol_stack: tun_smol_stack,
            packets: VecDeque::new()
        }
    }
}

pub struct TunSmolStack<'a, 'b: 'a, 'c: 'a + 'b> {
    pub sockets: SocketSet<'a, 'b, 'c>,
    current_key: usize,
    socket_handles: HashMap<usize, SocketHandle>,
    device: Option<TunDevice>,
    ip_addrs: Option<std::vec::Vec<IpCidr>>,
    default_v4_gw: Option<Ipv4Address>,
    default_v6_gw: Option<Ipv6Address>,
    pub interface: Option<Interface<'a, 'b, 'c, TunDevice>>,
}

impl<'a, 'b: 'a, 'c: 'a + 'b> TunSmolStack<'a, 'b, 'c> {
    pub fn new(interface_name: String) -> Box<TunSmolStack<'a, 'b, 'c>> {
        let socket_set = SocketSet::new(vec![]);
        let device = TunDevice::new(interface_name.as_str()).unwrap();
        let ip_addrs = std::vec::Vec::new();

        Box::new(TunSmolStack {
            sockets: socket_set,
            current_key: 0,
            socket_handles: HashMap::new(),
            device: Some(device),
            ip_addrs: Some(ip_addrs),
            default_v4_gw: None,
            default_v6_gw: None,
            interface: None,
        })
    }

    pub fn new_socket_handle_key(&mut self) -> usize {
        //TODO: panic when usize is about to overflow
        self.current_key += 1;
        self.current_key
    }

    pub fn add_socket(&mut self, socket_type: SocketType) -> usize {
        match socket_type {
            SocketType::TCP => {
                let rx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
                let tx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
                let socket = TcpSocket::new(rx_buffer, tx_buffer);
                let handle = self.sockets.add(socket);
                let handke_key = self.new_socket_handle_key();
                self.socket_handles.insert(handke_key, handle);
                handke_key
            }
            SocketType::UDP => {
                let rx_buffer = UdpSocketBuffer::new(Vec::new(), vec![0; 1024]);
                let tx_buffer = UdpSocketBuffer::new(Vec::new(), vec![0; 1024]);
                let socket = UdpSocket::new(rx_buffer, tx_buffer);
                let handle = self.sockets.add(socket);
                let handke_key = self.new_socket_handle_key();
                self.socket_handles.insert(handke_key, handle);
                handke_key
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
                panic! {"wrong choice for socket type"}
            }
        }
    }

    pub fn connect_ipv4(
        &mut self,
        socket_handle: usize,
        address: CIpv4Address,
        src_port: u16,
        dst_port: u16,
    ) -> u8 {
        let mut socket_handle_ = self.socket_handles.get(&socket_handle);
        match socket_handle_ {
            Some(socket_handle__) => {
                let mut socket = self.sockets.get::<TcpSocket>(*socket_handle__);
                let r = socket.connect(
                    (
                        Ipv4Address::new(
                            address.address[0],
                            address.address[1],
                            address.address[2],
                            address.address[3],
                        ),
                        dst_port,
                    ),
                    src_port,
                );
                match r {
                    Ok(_) => 0,
                    _ => 2,
                }
            }
            None => 1
        }
    }

    pub fn connect_ipv6(
        &mut self,
        socket_handle: usize,
        address: CIpv6Address,
        src_port: u16,
        dst_port: u16,
    ) -> u8 {
        let mut socket_handle_ = self.socket_handles.get(&socket_handle);
        match socket_handle_ {
            Some(socket_handle__) => {
                let mut socket = self.sockets.get::<TcpSocket>(*socket_handle__);
                let r = socket.connect(
                    (
                        Ipv6Address::new(
                            address.address[0],
                            address.address[1],
                            address.address[2],
                            address.address[3],
                            address.address[4],
                            address.address[5],
                            address.address[6],
                            address.address[7],
                        ),
                        dst_port,
                    ),
                    src_port,
                );
                match r {
                    Ok(_) => 0,
                    _ => 2,
                }
            }
            None => 1
        }
    }

    pub fn add_ipv4_address(&mut self, cidr: CIpv4Cidr) {
        self.ip_addrs.as_mut().unwrap().push(IpCidr::new(
            IpAddress::v4(
                cidr.address.address[0],
                cidr.address.address[1],
                cidr.address.address[2],
                cidr.address.address[3],
            ),
            cidr.prefix,
        ));
    }

    pub fn add_ipv6_address(&mut self, cidr: CIpv6Cidr) {
        self.ip_addrs.as_mut().unwrap().push(IpCidr::new(
            IpAddress::v6(
                cidr.address.address[0],
                cidr.address.address[1],
                cidr.address.address[2],
                cidr.address.address[3],
                cidr.address.address[4],
                cidr.address.address[5],
                cidr.address.address[6],
                cidr.address.address[7],
            ),
            cidr.prefix,
        ));
    }

    pub fn add_default_v4_gateway(&mut self, address: CIpv4Address) {
        self.default_v4_gw = Some(Ipv4Address::new(
            address.address[0],
            address.address[1],
            address.address[2],
            address.address[3],
        ));
    }

    pub fn add_default_v6_gateway(&mut self, address: CIpv6Address) {
        self.default_v6_gw = Some(Ipv6Address::new(
            address.address[0],
            address.address[1],
            address.address[2],
            address.address[3],
            address.address[4],
            address.address[5],
            address.address[6],
            address.address[7],
        ));
    }

    pub fn finalize(&mut self) -> u8 {
        let routes_storage = BTreeMap::new();
        let mut routes = Routes::new(routes_storage);
        //TODO: return C error if something is wrong, no unwrap
        routes
            .add_default_ipv4_route(self.default_v4_gw.unwrap())
            .unwrap();
        routes
            .add_default_ipv6_route(self.default_v6_gw.unwrap())
            .unwrap();
        let interface = InterfaceBuilder::new(self.device.take().unwrap())
            .ip_addrs(self.ip_addrs.take().unwrap())
            .routes(routes)
            .finalize();
        self.interface = Some(interface);
        0
    }
}
