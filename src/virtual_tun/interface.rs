extern crate rand;

use super::smol_stack::SmolSocket;
use super::smol_stack::{SmolStack, SocketType, Packet, Blob};
use super::virtual_tun::VirtualTunInterface as VirtualTunDevice;
use smoltcp::phy::TunInterface as TunDevice;
use smoltcp::socket::{SocketHandle, TcpSocket};
use smoltcp::time::Instant;
use smoltcp::wire::{IpAddress, IpCidr, IpEndpoint, Ipv4Address, Ipv6Address};
use std::ffi::{CStr, c_void};
use std::os::raw::{c_char, c_int};
use std::slice;
use std::str::{self};
use smoltcp::phy::wait as phy_wait;
use std::os::unix::io::AsRawFd;
use smoltcp::phy::TunInterface;


type OnPacketToOutside = unsafe extern "C" fn(data: *mut u8, len: usize, packet_type: u8) -> c_int;
static mut onPacketToOutside: Option<OnPacketToOutside> = None;

pub enum SmolSocketType {
    VirtualTun,
    Tun,
}

pub enum SmolStackType<'a, 'b: 'a, 'c: 'a + 'b, 'e> {
    VirtualTun(SmolStack<'a, 'b, 'c, 'e, VirtualTunDevice>),
    Tun(SmolStack<'a, 'b, 'c, 'e, TunDevice>),
}

impl<'a, 'b: 'a, 'c: 'a + 'b, 'e> SmolStackType<'a, 'b, 'c, 'e> {
    pub fn new_virtual_tun(interface_name: String) -> Box<SmolStackType<'a, 'b, 'c, 'e>> {
        let device = VirtualTunDevice::new(interface_name.as_str()).unwrap();
        let smol_stack = SmolStack::new(interface_name, device);
        Box::new(SmolStackType::VirtualTun(smol_stack))
    }

    pub fn new_tun(interface_name: String) -> Box<SmolStackType<'a, 'b, 'c, 'e>> {
        let device = TunDevice::new(interface_name.as_str()).unwrap();
        let smol_stack = SmolStack::new(interface_name, device);
        Box::new(SmolStackType::Tun(smol_stack))
    }

    pub fn new_socket_handle_key(&mut self) -> usize {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => {
                smol_stack.new_socket_handle_key()
            }
            &mut SmolStackType::Tun(ref mut smol_stack) => smol_stack.new_socket_handle_key(),
        }
    }

    pub fn add_socket(&mut self, socket_type: SocketType) -> usize {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => {
                smol_stack.add_socket(socket_type)
            }
            &mut SmolStackType::Tun(ref mut smol_stack) => smol_stack.add_socket(socket_type),
        }
    }

    pub fn tcp_connect_ipv4(
        &mut self,
        socket_handle_key: usize,
        address: CIpv4Address,
        src_port: u16,
        dst_port: u16,
    ) -> u8 {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => {
                smol_stack.tcp_connect_ipv4(socket_handle_key, address, src_port, dst_port)
            }
            &mut SmolStackType::Tun(ref mut smol_stack) => {
                smol_stack.tcp_connect_ipv4(socket_handle_key, address, src_port, dst_port)
            }
        }
    }

    pub fn get_smol_socket(&mut self, socket_handle_key: usize) -> Option<&mut SmolSocket<'e>> {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => {
                smol_stack.get_smol_socket(socket_handle_key)
            }
            &mut SmolStackType::Tun(ref mut smol_stack) => {
                smol_stack.get_smol_socket(socket_handle_key)
            }
        }
    }

    pub fn tcp_connect_ipv6(
        &mut self,
        socket_handle_key: usize,
        address: CIpv6Address,
        src_port: u16,
        dst_port: u16,
    ) -> u8 {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => {
                smol_stack.tcp_connect_ipv6(socket_handle_key, address, src_port, dst_port)
            }
            &mut SmolStackType::Tun(ref mut smol_stack) => {
                smol_stack.tcp_connect_ipv6(socket_handle_key, address, src_port, dst_port)
            }
        }
    }

    pub fn add_ipv4_address(&mut self, cidr: CIpv4Cidr) {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => smol_stack.add_ipv4_address(cidr),
            &mut SmolStackType::Tun(ref mut smol_stack) => smol_stack.add_ipv4_address(cidr),
        }
    }

    pub fn add_ipv6_address(&mut self, cidr: CIpv6Cidr) {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => smol_stack.add_ipv6_address(cidr),
            &mut SmolStackType::Tun(ref mut smol_stack) => smol_stack.add_ipv6_address(cidr),
        }
    }

    pub fn add_default_v4_gateway(&mut self, address: CIpv4Address) {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => {
                smol_stack.add_default_v4_gateway(address)
            }
            &mut SmolStackType::Tun(ref mut smol_stack) => {
                smol_stack.add_default_v4_gateway(address)
            }
        }
    }

    pub fn add_default_v6_gateway(&mut self, address: CIpv6Address) {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => {
                smol_stack.add_default_v6_gateway(address)
            }
            &mut SmolStackType::Tun(ref mut smol_stack) => {
                smol_stack.add_default_v6_gateway(address)
            }
        }
    }

    pub fn finalize(&mut self) -> u8 {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => smol_stack.finalize(),
            &mut SmolStackType::Tun(ref mut smol_stack) => smol_stack.finalize(),
        }
    }

    pub fn poll(&mut self) -> u8 {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => smol_stack.poll(),
            &mut SmolStackType::Tun(ref mut smol_stack) => smol_stack.poll(),
        }
    }

    pub fn spin(&mut self, socket_handle_key: usize) -> u8 {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => {
                smol_stack.spin(socket_handle_key)
            }
            &mut SmolStackType::Tun(ref mut smol_stack) => smol_stack.spin(socket_handle_key),
        }
    }

    pub fn phy_wait(&mut self) {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => {
                //phy_wait(smol_stack.device.unwrap().as_raw_fd(), smol_stack.interface.unwrap().poll_delay(&smol_stack.sockets, timestamp)).expect("wait error")
            }
            &mut SmolStackType::Tun(ref mut smol_stack) => {
                phy_wait(smol_stack.device.unwrap().as_raw_fd(), smol_stack.interface.unwrap().poll_delay(&smol_stack.sockets, timestamp)).expect("wait error")
            }
        }
    }


}

#[repr(C)]
pub struct CIpv4Address {
    pub address: [u8; 4],
}

impl Into<Ipv4Address> for CIpv4Address {
    fn into(self) -> Ipv4Address {
        Ipv4Address::new(
            self.address[0],
            self.address[1],
            self.address[2],
            self.address[3],
        )
    }
}

impl Into<IpAddress> for CIpv4Address {
    fn into(self) -> IpAddress {
        IpAddress::v4(
            self.address[0],
            self.address[1],
            self.address[2],
            self.address[3],
        )
    }
}

#[repr(C)]
pub struct CIpv6Address {
    pub address: [u16; 8],
}

impl Into<Ipv6Address> for CIpv6Address {
    fn into(self) -> Ipv6Address {
        Ipv6Address::new(
            self.address[0],
            self.address[1],
            self.address[2],
            self.address[3],
            self.address[4],
            self.address[5],
            self.address[6],
            self.address[7],
        )
    }
}

impl Into<IpAddress> for CIpv6Address {
    fn into(self) -> IpAddress {
        IpAddress::v6(
            self.address[0],
            self.address[1],
            self.address[2],
            self.address[3],
            self.address[4],
            self.address[5],
            self.address[6],
            self.address[7],
        )
    }
}

//Keep synced with CIpEndpointType on interface.h
static CIpEndpoint_NONE: u8 = 0;
static CIPENDPOINT_IPV4: u8 = 1;
static CIPENDPOINT_IPV6: u8 = 0;

#[repr(C)]
pub struct CIpEndpoint {
    pub endpoint_type: u8,
    pub ipv4: CIpv4Address,
    pub ipv6: CIpv6Address,
    pub port: u16,
}

impl Into<Option<IpEndpoint>> for CIpEndpoint {
    fn into(self) -> Option<IpEndpoint> {
        if self.endpoint_type == CIPENDPOINT_IPV4 {
            Some(IpEndpoint::new(
                IpAddress::v4(
                    self.ipv4.address[0],
                    self.ipv4.address[1],
                    self.ipv4.address[2],
                    self.ipv4.address[3],
                ),
                self.port,
            ))
        } else if self.endpoint_type == CIPENDPOINT_IPV6 {
            Some(IpEndpoint::new(
                IpAddress::v6(
                    self.ipv6.address[0],
                    self.ipv6.address[1],
                    self.ipv6.address[2],
                    self.ipv6.address[3],
                    self.ipv6.address[4],
                    self.ipv6.address[5],
                    self.ipv6.address[6],
                    self.ipv6.address[7],
                ),
                self.port,
            ))
        } else {
            None
        }
    }
}

#[repr(C)]
pub struct CIpv4Cidr {
    pub address: CIpv4Address,
    pub prefix: u8,
}

#[repr(C)]
pub struct CIpv6Cidr {
    pub address: CIpv6Address,
    pub prefix: u8,
}

#[no_mangle]
pub extern "C" fn registerOnPacketToOutside(callback: Option<OnPacketToOutside>) -> c_int {
    unsafe {
        onPacketToOutside = callback;
    }
    0
}

#[no_mangle]
pub extern "C" fn smol_stack_smol_stack_new_virtual_tun<'a, 'b: 'a, 'c: 'a + 'b, 'e>(
    interface_name: *const c_char,
) -> Box<SmolStackType<'a, 'b, 'c, 'e>> {
    let interface_name_c_str: &CStr = unsafe { CStr::from_ptr(interface_name) };
    let interface_name_slice: &str = interface_name_c_str.to_str().unwrap();
    let s: String = interface_name_slice.to_owned();
    SmolStackType::new_virtual_tun(s)
}

#[no_mangle]
pub extern "C" fn smol_stack_smol_stack_new_tun<'a, 'b: 'a, 'c: 'a + 'b, 'e>(
    interface_name: *const c_char,
) -> Box<SmolStackType<'a, 'b, 'c, 'e>> {
    let interface_name_c_str: &CStr = unsafe { CStr::from_ptr(interface_name) };
    let interface_name_slice: &str = interface_name_c_str.to_str().unwrap();
    let s: String = interface_name_slice.to_owned();
    SmolStackType::new_tun(s)
}

#[no_mangle]
pub extern "C" fn smol_stack_smol_socket_send(
    smol_stack: &mut SmolStackType,
    socket_handle_key: usize,
    data: *mut u8,
    len: usize,
    endpoint: CIpEndpoint,
    pointer_to_owner: *const c_void,
    pointer_to_destructor: unsafe extern "C" fn(*const c_void) -> u8
) -> u8 {
    let smol_socket = smol_stack.get_smol_socket(socket_handle_key);
    let packet_as_slice = unsafe { slice::from_raw_parts(data, len) };
    let packet = Packet {
        blob: Blob{
            slice: packet_as_slice, 
            start: 0,
            pointer_to_owner: pointer_to_owner, 
            pointer_to_destructor: pointer_to_destructor
        },
        endpoint: Into::<Option<IpEndpoint>>::into(endpoint),
    };
    match smol_socket {
        Some(smol_socket_) => {
            smol_socket_.send(packet);
            0
        }
        None => 1,
    }
}

//packets (ethernet, ip, tcp, etc) from the world to the stack
pub extern "C" fn smol_stack_receive_packet(data: *mut u8, len: usize, packet_type: u8) {}

#[no_mangle]
pub extern "C" fn smol_stack_add_socket(smol_stack: &mut SmolStackType, socket_type: u8) -> usize {
    match socket_type {
        0 => smol_stack.add_socket(SocketType::TCP),
        1 => smol_stack.add_socket(SocketType::UDP),
        _ => panic!("wrong type"),
    }
}

#[no_mangle]
pub extern "C" fn smol_stack_phy_wait(smol_stack: &mut SmolStackType, socket_type: u8) {
    match socket_type {
        0 => smol_stack.phy_wait(),
        1 => smol_stack.phy_wait(),
        _ => panic!("wrong type"),
    }
}



#[no_mangle]
pub extern "C" fn smol_stack_tcp_connect_ipv4(
    smol_stack: &mut SmolStackType,
    socket_handle_key: usize,
    address: CIpv4Address,
    src_port: u16,
    dst_port: u16,
) -> u8 {
    println!("smol_stack_tcp_connect_ipv4 handle {}", socket_handle_key);

    smol_stack.tcp_connect_ipv4(socket_handle_key, address, src_port, dst_port)
}

#[no_mangle]
pub extern "C" fn smol_stack_tcp_connect_ipv6(
    smol_stack: &mut SmolStackType,
    socket_handle_key: usize,
    address: CIpv6Address,
    src_port: u16,
    dst_port: u16,
) -> u8 {
    smol_stack.tcp_connect_ipv6(socket_handle_key, address, src_port, dst_port)
}

#[no_mangle]
pub extern "C" fn smol_stack_poll(smol_stack: &mut SmolStackType) -> u8 {
    smol_stack.poll()
}

#[no_mangle]
pub extern "C" fn smol_stack_spin(smol_stack: &mut SmolStackType, socket_handle_key: usize) -> u8 {
    smol_stack.spin(socket_handle_key)
}

#[no_mangle]
pub extern "C" fn smol_stack_add_ipv4_address(smol_stack: &mut SmolStackType, cidr: CIpv4Cidr) {
    smol_stack.add_ipv4_address(cidr);
}

#[no_mangle]
pub extern "C" fn smol_stack_add_ipv6_address(smol_stack: &mut SmolStackType, cidr: CIpv6Cidr) {
    smol_stack.add_ipv6_address(cidr);
}

#[no_mangle]
pub extern "C" fn smol_stack_add_default_v4_gateway(
    smol_stack: &mut SmolStackType,
    address: CIpv4Address,
) {
    smol_stack.add_default_v4_gateway(address);
}

#[no_mangle]
pub extern "C" fn smol_stack_add_default_v6_gateway(
    smol_stack: &mut SmolStackType,
    address: CIpv6Address,
) {
    smol_stack.add_default_v6_gateway(address);
}

#[no_mangle]
pub extern "C" fn smol_stack_finalize<'a, 'b: 'a, 'c: 'a + 'b, 'e>(
    smol_stack: &mut SmolStackType<'a, 'b, 'c, 'e>,
) -> u8 {
    smol_stack.finalize()
}
