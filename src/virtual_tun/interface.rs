extern crate rand;

use std::str::{self};
use std::os::raw::{c_int, c_char};
use std::ffi::CStr;
use smoltcp::socket::{SocketHandle, TcpSocket};
use super::smol_stack::{TunSmolStack, SocketType};
use smoltcp::time::Instant;
use smoltcp::wire::{Ipv4Address, Ipv6Address, IpAddress,IpCidr};

type OnPacketToOutside = unsafe extern "C" fn(data: *mut u8, len: usize, packet_type: u8) -> c_int;
static mut onPacketToOutside: Option<OnPacketToOutside> = None;

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

#[repr(C)]
pub struct CIpv4Cidr {
    pub address: CIpv4Address,
    pub prefix: u8
}

#[repr(C)]
pub struct CIpv6Cidr {
    pub address: CIpv6Address,
    pub prefix: u8
}

#[no_mangle]
pub extern "C" fn registerOnPacketToOutside(callback: Option<OnPacketToOutside>) -> c_int
{
    unsafe{onPacketToOutside = callback;}
    0
}

#[no_mangle]
pub extern "C" fn smol_stack_tun_smol_stack_new<'a, 'b: 'a, 'c: 'a + 'b>(interface_name: *const c_char) -> Box<TunSmolStack<'a, 'b, 'c>> {
    let interface_name_c_str: &CStr = unsafe { CStr::from_ptr(interface_name) };
    let interface_name_slice: &str = interface_name_c_str.to_str().unwrap();
    let s: String = interface_name_slice.to_owned();
    TunSmolStack::new(s)
}

#[no_mangle]
pub extern "C" fn smol_stack_tun_smol_socket_send(tun_smol_stack: &mut TunSmolStack, socket_handle_key: usize, data: *mut u8, len: usize) -> u8 {
    let smol_socket = tun_smol_stack.get_smol_socket(socket_handle_key);
    match smol_socket {
        Some(smol_socket_) => {
            smol_socket_.send(data, len);
            0
        }
        None => 1
    }
}

//packets (ethernet, ip, tcp, etc) from the world to the stack
pub extern "C" fn smol_stack_tun_receive_packet(data: *mut u8, len: usize, packet_type: u8) {
    
}

#[no_mangle]
pub extern "C" fn smol_stack_add_socket(tun_smol_stack: &mut TunSmolStack, socket_type: u8) -> usize  {
    match socket_type {
        0 => tun_smol_stack.add_socket(SocketType::TCP),
        1 => tun_smol_stack.add_socket(SocketType::UDP),
        _ => panic!("wrong type")
    }
}

#[no_mangle]
pub extern "C" fn smol_stack_tcp_connect_ipv4(tun_smol_stack: &mut TunSmolStack, socket_handle_key: usize, address: CIpv4Address, src_port: u16, dst_port: u16) -> u8 {
    tun_smol_stack.tcp_connect_ipv4(socket_handle_key, address, src_port, dst_port)
}

#[no_mangle]
pub extern "C" fn smol_stack_tcp_connect_ipv6(tun_smol_stack: &mut TunSmolStack, socket_handle_key: usize, address: CIpv6Address, src_port: u16, dst_port: u16) -> u8 {
    tun_smol_stack.tcp_connect_ipv6(socket_handle_key, address, src_port, dst_port)
}

#[no_mangle]
pub extern "C" fn smol_stack_spin(tun_smol_stack: &mut TunSmolStack, socket_handle_key: usize) {
    let timestamp = Instant::now();
    
    match tun_smol_stack.interface.as_mut().unwrap().poll(&mut tun_smol_stack.sockets, timestamp) {
        Ok(_) => {},
        Err(e) => {
            //debug!("poll error: {}",e);
        }
    }

    //let mut socket = tun_smol_stack.sockets.get(*socket_handle);
    //let local_port = 49152 + rand::random::<u16>() % 16384;
    //socket.connect((Ipv4Address::new(172, 217, 29, 14), 80), local_port).unwrap();
    /*
    state = match state {
        State::Connect if !socket.is_active() => {
            debug!("connecting");
            let local_port = 49152 + rand::random::<u16>() % 16384;
            socket.connect((address, url.port().unwrap_or(80)), local_port).unwrap();
            State::Request
        }
    }
    */
}

#[no_mangle]
pub extern "C" fn smol_stack_add_ipv4_address(tun_smol_stack: &mut TunSmolStack, cidr: CIpv4Cidr) {
    tun_smol_stack.add_ipv4_address(cidr);
}

#[no_mangle]
pub extern "C" fn smol_stack_add_ipv6_address(tun_smol_stack: &mut TunSmolStack, cidr: CIpv6Cidr) {
    tun_smol_stack.add_ipv6_address(cidr);
}

#[no_mangle]
pub extern "C" fn smol_stack_add_default_v4_gateway(tun_smol_stack: &mut TunSmolStack, address: CIpv4Address) {
    tun_smol_stack.add_default_v4_gateway(address);
}

#[no_mangle]
pub extern "C" fn smol_stack_add_default_v6_gateway(tun_smol_stack: &mut TunSmolStack, address: CIpv6Address) {
    tun_smol_stack.add_default_v6_gateway(address);
}

#[no_mangle]
pub extern "C" fn smol_stack_finalize<'a, 'b: 'a, 'c: 'a + 'b>(tun_smol_stack: &mut TunSmolStack<'a, 'b, 'c>) -> u8 {
    tun_smol_stack.finalize()
}