//use smoltcp_openvpn_bridge::virtual_tun::VirtualTunInterface;
use super::interface::{CBuffer, CIpv4Address, CIpv4Cidr, CIpv6Address, CIpv6Cidr};
use super::virtual_tun::VirtualTunInterface as TunDevice;
use smoltcp::iface::{Interface, InterfaceBuilder, Routes};
use smoltcp::phy::wait as phy_wait;
use smoltcp::phy::{self, Device};
use smoltcp::socket::{
    AnySocket, RawSocket, RawSocketBuffer, Socket, SocketHandle, SocketRef, SocketSet, TcpSocket,
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
use std::ffi::c_void;
use std::rc::Rc;
use std::slice;
use std::sync::{Arc, Mutex};

#[derive(PartialEq, Clone)]
pub enum SocketType {
    RAW_IPV4,
    RAW_IPV6,
    ICMP,
    TCP,
    UDP,
}

pub struct Blob<'a> {
    pub slice: &'a [u8],
    pub start: usize,
    //A pointer do the object (SmolOwner in C++) that owns the data on the slice
    pub pointer_to_owner: *const c_void,
    /*
        Function pointer to the function that receives the pointer_to_owner
        and deletes it, thus callings its destructor which deletes the owner
        of the data on the slice, which deletes the data on the slice
    */
    pub pointer_to_destructor: unsafe extern "C" fn(*const c_void) -> u8,
}

pub struct Packet<'a> {
    pub blob: Blob<'a>,
    pub endpoint: Option<IpEndpoint>,
}

impl<'a> Drop for Blob<'a> {
    fn drop(&mut self) {
        println!("blob drop!");
        let f = self.pointer_to_destructor;
        let r = unsafe { f(self.pointer_to_owner) };
        println!("blob drop result: {}", r);
    }
}

pub struct SmolSocket<'a> {
    pub socket_type: SocketType,
    //Socket number inside SmolStack
    pub socket_handle: SocketHandle,
    pub to_send: Arc<Mutex<VecDeque<Packet<'a>>>>,
    //If we couldn't send entire packet at once, hold it here for next send
    current_to_send: Option<Packet<'a>>,
    pub received: Arc<Mutex<VecDeque<&'a [u8]>>>,
}

impl<'a> SmolSocket<'a> {
    pub fn new(socket_handle: SocketHandle, socket_type: SocketType) -> SmolSocket<'a> {
        SmolSocket {
            socket_type: socket_type,
            socket_handle: socket_handle,
            to_send: Arc::new(Mutex::new(VecDeque::new())),
            current_to_send: None,
            received: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn send(&mut self, packet: Packet<'a>) -> u8 {
        if packet.endpoint.is_none()
            && (self.socket_type == SocketType::UDP || self.socket_type == SocketType::ICMP)
        {
            panic!("this socket type needs an endpoint to send to");
        }
        self.to_send.lock().unwrap().push_back(packet);
        0
    }

    //TODO: figure out a better way than copying
    pub fn receive(&mut self) -> CBuffer {
        let s: &'a [u8];
        {
            //Create a scope so we hold the queue for the least ammount needed
            //TODO: do I really need to create a scope?
            s = self.received.lock().unwrap().pop_front().unwrap();
        }
        let ss = s.as_mut_ptr();
        CBuffer {
            data: ss,
            len: s.len(),
        }
    }

    pub fn get_latest_packet(&mut self) -> Option<Packet> {
        //If the last step couldn't send the entire blob,
        //the packet is in `self.current_to_send`, so we return it again
        //otherwise we return a fresh packet from the queue
        match self.current_to_send.take() {
            Some(packet) => Some(packet),
            //TODO: verify assertion below
            //lock happens very birefly, so the list is not kept locked much time
            None => {
                let packet = self.to_send.lock().unwrap().pop_front();
                packet
            }
        }
    }
}

pub struct SmolStack<'a, 'b: 'a, 'c: 'a + 'b, DeviceT>
where
    DeviceT: for<'d> Device<'d>,
{
    pub sockets: SocketSet<'a, 'b, 'c>,
    current_key: usize,
    smol_sockets: HashMap<usize, SmolSocket<'a>>,
    pub device: Option<DeviceT>,
    ip_addrs: Option<std::vec::Vec<IpCidr>>,
    default_v4_gw: Option<Ipv4Address>,
    default_v6_gw: Option<Ipv6Address>,
    pub interface: Option<Interface<'a, 'b, 'c, DeviceT>>,
}

impl<'a, 'b: 'a, 'c: 'a + 'b, DeviceT> SmolStack<'a, 'b, 'c, DeviceT>
where
    DeviceT: for<'d> Device<'d>,
{
    pub fn new(interface_name: String, device: DeviceT) -> SmolStack<'a, 'b, 'c, DeviceT> {
        let socket_set = SocketSet::new(vec![]);
        let ip_addrs = std::vec::Vec::new();

        SmolStack {
            sockets: socket_set,
            current_key: 0,
            smol_sockets: HashMap::new(),
            device: Some(device),
            ip_addrs: Some(ip_addrs),
            default_v4_gw: None,
            default_v6_gw: None,
            interface: None,
        }
    }

    pub fn new_socket_handle_key(&mut self) -> usize {
        //TODO: panic when usize is about to overflow
        self.current_key += 1;
        self.current_key
    }

    pub fn add_socket(&mut self, socket_type: SocketType, smol_socket_handle: usize) -> u8 {
        match socket_type {
            SocketType::TCP => {
                let rx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
                let tx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
                let socket = TcpSocket::new(rx_buffer, tx_buffer);
                let handle = self.sockets.add(socket);
                //let handke_key = self.new_socket_handle_key();
                let smol_socket = SmolSocket::new(handle, SocketType::TCP);
                self.smol_sockets.insert(smol_socket_handle, smol_socket);
                0
            }
            SocketType::UDP => {
                let rx_buffer = UdpSocketBuffer::new(Vec::new(), vec![0; 1024]);
                let tx_buffer = UdpSocketBuffer::new(Vec::new(), vec![0; 1024]);
                let socket = UdpSocket::new(rx_buffer, tx_buffer);
                let handle = self.sockets.add(socket);
                //let handke_key = self.new_socket_handle_key();
                let smol_socket = SmolSocket::new(handle, SocketType::UDP);
                self.smol_sockets.insert(smol_socket_handle, smol_socket);
                0
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

    pub fn get_smol_socket(&mut self, smol_socket_handle: usize) -> Option<&mut SmolSocket<'a>> {
        let smol_socket = self.smol_sockets.get_mut(&smol_socket_handle);
        smol_socket
    }

    pub fn tcp_connect_ipv4(
        &mut self,
        smol_socket_handle: usize,
        address: CIpv4Address,
        src_port: u16,
        dst_port: u16,
    ) -> u8 {
        println!(
            "gonna get smol socket with handle key {}",
            smol_socket_handle
        );
        let smol_socket_ = self.smol_sockets.get(&smol_socket_handle);
        match smol_socket_ {
            Some(smol_socket) => {
                let socket_handle = smol_socket.socket_handle;
                let mut socket = self.sockets.get::<TcpSocket>(socket_handle);
                let r = socket.connect((Into::<Ipv4Address>::into(address), dst_port), src_port);
                match r {
                    Ok(_) => {
                        println!("connection ok");
                        0
                    }
                    _ => {
                        println!("connection error");
                        2
                    }
                }
            }
            None => {
                println!("NO smol socket");
                1
            }
        }
    }

    pub fn tcp_connect_ipv6(
        &mut self,
        smol_socket_handle: usize,
        address: CIpv6Address,
        src_port: u16,
        dst_port: u16,
    ) -> u8 {
        let smol_socket_ = self.smol_sockets.get(&smol_socket_handle);
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

    pub fn poll(&mut self) -> u8 {
        let timestamp = Instant::now();
        match self
            .interface
            .as_mut()
            .unwrap()
            .poll(&mut self.sockets, timestamp)
        {
            Ok(_) => 0,
            Err(e) => {
                //debug!("poll error: {}",e);
                1
            }
        }
    }

    pub fn spin(&mut self, smol_socket_handle: usize) -> u8 {
        let smol_socket = self.smol_sockets.get_mut(&smol_socket_handle).unwrap();
        //println!("spin");
        match smol_socket.socket_type {
            SocketType::TCP => {
                let mut socket = self.sockets.get::<TcpSocket>(smol_socket.socket_handle);
                if socket.may_send() {
                    let packet = smol_socket.get_latest_packet();
                    /*
                    let bytes_sent = socket.send_slice(packet.unwrap().slice);
                    match bytes_sent {
                        Ok(_) => 0,
                        Err(e) => 1,
                    }
                    */

                    //let packet = Some(Packet{endpoint: None, slice: &[], socket_type: SocketType::ICMP});
                    match packet {
                        Some(packet) => {
                            println!("some packet");
                            use std::str;
                            if let Ok(s) = str::from_utf8(packet.blob.slice) {
                                println!("{}", s);
                            }
                            let bytes_sent = socket.send_slice(packet.blob.slice);
                            
                            match bytes_sent {
                                Ok(b) => {
                                    println!("sent {} bytes", b);
                                    //Sent less than entire packet, so we must put this packet
                                    //in `smol_socket.current_to_send` so it's returned the next time
                                    //so we can continue sending it
                                    if b < packet.blob.slice.len() {
                                        //smol_socket.current_to_send = Some(packet);
                                        //0
                                    } else {
                                        //Sent the entire packet, nothing needs to be done
                                        //0
                                    }
                                }
                                Err(e) => {
                                    println!("bytes not sent, ERROR {}, putting packet back", e);
                                    //smol_socket.current_to_send = Some(packet);
                                    //1
                                }
                            }
                        }
                        None => {
                            //println!("NO packet");
                            //1
                        }
                    }
                } else {
                    //1
                }
                if socket.can_recv() {
                    socket
                        .recv(|data| {
                            let len = data.len();
                            //println!("{}", str::from_utf8(data).unwrap_or("(invalid utf8)"));
                            smol_socket.received.lock().unwrap().push_back(data);
                            //smol_socket.receive(data);
                            (len, ())
                        })
                        .unwrap();
                //0
                } else {
                    //2
                }
                0
            }
            SocketType::UDP => {
                let mut socket = self.sockets.get::<UdpSocket>(smol_socket.socket_handle);
                let packet = smol_socket.get_latest_packet().unwrap();
                //TODO: send correct slice
                let bytes_sent = socket.send_slice(packet.blob.slice, packet.endpoint.unwrap());
                match bytes_sent {
                    Ok(_) => 0,
                    Err(e) => 1,
                }
            }
            //TODO
            SocketType::ICMP => 0,
            SocketType::RAW_IPV4 => 0,
            SocketType::RAW_IPV6 => 0,
        }
    }
}
