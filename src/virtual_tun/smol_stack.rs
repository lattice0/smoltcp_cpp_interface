//use smoltcp_openvpn_bridge::virtual_tun::VirtualTunInterface;
use super::interface::{CIpv4Address, CIpv4Cidr, CIpv6Address, CIpv6Cidr};
use super::virtual_tun::VirtualTunInterface as TunDevice;
use smoltcp::iface::{Interface, InterfaceBuilder, Routes};
use smoltcp::phy::{self, Device};
use smoltcp::socket::{
    SocketRef, AnySocket, RawSocket, RawSocketBuffer, Socket, SocketHandle, SocketSet, TcpSocket,
    TcpSocketBuffer, UdpSocket, UdpSocketBuffer,
};
use smoltcp::storage::PacketMetadata;
use smoltcp::time::Instant;
use smoltcp::wire::{
    IpAddress, IpCidr, IpEndpoint, IpProtocol, IpVersion, Ipv4Address, Ipv6Address,
};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::slice;

#[derive(PartialEq, Clone)]
pub enum SocketType {
    RAW_IPV4,
    RAW_IPV6,
    ICMP,
    TCP,
    UDP,
}

pub struct Blob {
    data: *mut u8,
    len: usize,
    start: usize,
}

pub struct Packet {
    socket_type: SocketType,
    blob: Blob,
    endpoint: Option<IpEndpoint>
}

impl Drop for Blob {
    fn drop(&mut self) {
        //
    }
}

pub struct SmolSocket {
    pub socket_type: SocketType,
    pub socket_handle: SocketHandle,
    pub packets: Arc<Mutex<VecDeque<Packet>>>,
    //If we couldn't send entire packet at once, hold it here for next send
    current: Option<Packet>
}

impl SmolSocket {
    pub fn new(socket_handle: SocketHandle, socket_type: SocketType) -> SmolSocket {
        SmolSocket {
            socket_type: socket_type,
            socket_handle: socket_handle,
            packets: Arc::new(Mutex::new(VecDeque::new())),
            current: None
        }
    }

    pub fn send(&mut self, data: *mut u8, len: usize, endpoint: Option<IpEndpoint>) -> u8 {
        if endpoint.is_none() && (self.socket_type == SocketType::UDP ||
        self.socket_type == SocketType::ICMP) {
            panic!("this socket type needs an endpoint to send to");
        }
        let packet = Packet {
            socket_type: self.socket_type.clone(),
            blob: Blob {
                data: data,
                len: len,
                start: 0
            },
            endpoint: endpoint
        };
        self.packets.lock().unwrap().push_back(packet);
        0
    }

    pub fn get_latest_packet(&mut self) -> Option<Packet> {
        //If the last step couldn't send the entire blob,
        //the packet is in `self.current`, so we return it again
        //otherwise we return a fresh packet from the queue
        match self.current.take() {
            Some(packet) => Some(packet),
            None => self.packets.lock().unwrap().pop_front()
        }
        
    }
}

pub struct TunSmolStack<'a, 'b: 'a, 'c: 'a + 'b> {
    pub sockets: SocketSet<'a, 'b, 'c>,
    current_key: usize,
    smol_sockets: HashMap<usize, SmolSocket>,
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
            smol_sockets: HashMap::new(),
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
                let smol_socket = SmolSocket::new(handle, SocketType::TCP);
                self.smol_sockets.insert(handke_key, smol_socket);
                handke_key
            }
            SocketType::UDP => {
                let rx_buffer = UdpSocketBuffer::new(Vec::new(), vec![0; 1024]);
                let tx_buffer = UdpSocketBuffer::new(Vec::new(), vec![0; 1024]);
                let socket = UdpSocket::new(rx_buffer, tx_buffer);
                let handle = self.sockets.add(socket);
                let handke_key = self.new_socket_handle_key();
                let smol_socket = SmolSocket::new(handle, SocketType::UDP);
                self.smol_sockets.insert(handke_key, smol_socket);
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

    pub fn get_smol_socket(&mut self, socket_handle_key: usize) -> Option<&mut SmolSocket> {
        let smol_socket = self.smol_sockets.get_mut(&socket_handle_key);
        smol_socket
    }

    pub fn tcp_connect_ipv4(
        &mut self,
        socket_handle_key: usize,
        address: CIpv4Address,
        src_port: u16,
        dst_port: u16,
    ) -> u8 {
        let smol_socket_ = self.smol_sockets.get(&socket_handle_key);
        match smol_socket_ {
            Some(smol_socket) => {
                let socket_handle = smol_socket.socket_handle;
                let mut socket = self.sockets.get::<TcpSocket>(socket_handle);
                let r = socket.connect((Into::<Ipv4Address>::into(address), dst_port), src_port);
                match r {
                    Ok(_) => 0,
                    _ => 2,
                }
            }
            None => 1,
        }
    }

    pub fn tcp_connect_ipv6(
        &mut self,
        socket_handle_key: usize,
        address: CIpv6Address,
        src_port: u16,
        dst_port: u16,
    ) -> u8 {
        let smol_socket_ = self.smol_sockets.get(&socket_handle_key);
        match smol_socket_ {
            Some(smol_socket) => {
                let socket_handle = smol_socket.socket_handle;
                let mut socket = self.sockets.get::<TcpSocket>(socket_handle);
                let r = socket.connect((Into::<Ipv6Address>::into(address), dst_port), src_port);
                match r {
                    Ok(_) => 0,
                    _ => 2,
                }
            }
            None => 1,
        }
    }

    pub fn add_ipv4_address(&mut self, cidr: CIpv4Cidr) {
        self.ip_addrs.as_mut().unwrap().push(IpCidr::new(
            Into::<IpAddress>::into(cidr.address),
            cidr.prefix,
        ));
    }

    pub fn add_ipv6_address(&mut self, cidr: CIpv6Cidr) {
        self.ip_addrs.as_mut().unwrap().push(IpCidr::new(
            Into::<IpAddress>::into(cidr.address),
            cidr.prefix,
        ));
    }

    pub fn add_default_v4_gateway(&mut self, address: CIpv4Address) {
        self.default_v4_gw = Some(Into::<Ipv4Address>::into(address));
    }

    pub fn add_default_v6_gateway(&mut self, address: CIpv6Address) {
        self.default_v6_gw = Some(Into::<Ipv6Address>::into(address));
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

    pub fn poll(&mut self) -> u8{
        let timestamp = Instant::now();
        match self.interface.as_mut().unwrap().poll(&mut self.sockets, timestamp) {
            Ok(_) => 0,
            Err(e) => {
                //debug!("poll error: {}",e);
                1
            }
        }
    }


    pub fn spin(&mut self, socket_handle_key: usize) -> u8{
        let smol_socket = self.smol_sockets.get_mut(&socket_handle_key).unwrap();
        match smol_socket.socket_type {
            SocketType::TCP => {
                let mut socket = self.sockets.get::<TcpSocket>(smol_socket.socket_handle);
                let packet = smol_socket.get_latest_packet().unwrap();
                let packet_as_slice =  unsafe { slice::from_raw_parts(packet.blob.data, packet.blob.len) };
                //TODO: send correct slice
                let bytes_sent = socket.send_slice(packet_as_slice);
                match bytes_sent {
                    Ok(b) => {
                        //Sent less than entire packet, so we must put this packet 
                        //in `smol_socket.current` so it's returned the next time
                        //so we can continue sending it
                        if b < packet.blob.len {
                            smol_socket.current = Some(packet);                            
                            0
                        } else {
                            //Sent the entire packet, nothing needs to be done
                            0
                        }
                    }
                    Err(e) => {
                        1
                    }
                }
            }
            SocketType::UDP => {
                let mut socket = self.sockets.get::<UdpSocket>(smol_socket.socket_handle);
                let packet = smol_socket.get_latest_packet().unwrap();
                let packet_as_slice =  unsafe { slice::from_raw_parts(packet.blob.data, packet.blob.len) };
                //TODO: send correct slice
                let bytes_sent = socket.send_slice(packet_as_slice, packet.endpoint.unwrap());
                match bytes_sent {
                    Ok(_) => {
                        0
                    }
                    Err(e) => {
                        1
                    }
                }
            }
            //TODO
            SocketType::ICMP => {
                0
            }
            SocketType::RAW_IPV4 => {
                0
            }
            SocketType::RAW_IPV6 => {
                0
            }
        }
        /*
        let timestamp = Instant::now();
        match self.interface.as_mut().unwrap().poll(&mut self.sockets, timestamp)
        {
            Ok(_) => {
                
                sockets.iter_mut().filter_map(TcpSocket::downcast);
                let socket_interface: Box<dyn SocketInterface>;
                for socket in self.sockets.iter_mut() {
                    socket_interface = Box::new(socket.socket.unwrap());
                    if socket.may_send() {
                        socket.send_slice(b"\r\n").expect("cannot send");
                    }
                    if socket.can_recv() => {
                        socket.recv(|data| {
                            println!("{}", str::from_utf8(data).unwrap_or("(invalid utf8)"));
                            (data.len(), ())
                        }).unwrap();
                        State::Response
                    }
                }
                
            }
            Err(e) => {
                //debug!("poll error: {}",e);
            }
            
        }
        */
    }
}
