extern crate rand;

use std::str::{self};
use std::os::raw::{c_int};
use smoltcp::socket::{SocketHandle, TcpSocket};
use super::smol_stack::{TunSmolStack, SocketType};
use smoltcp::time::Instant;
use smoltcp::wire::{Ipv4Address, Ipv6Address, IpAddress, IpCidr};

type OnDataCallback = unsafe extern "C" fn(data: *mut u8, len: usize) -> c_int;
static mut onDataCallback_: Option<OnDataCallback> = None;

/*
#[repr(C)]
pub struct CSocketHandleResult {
    pub ok: bool,
    pub handle: usize,
}
*/

#[repr(C)]
pub struct CIpv4Address {
    pub address: [u8; 4],
}

#[repr(C)]
pub struct CIpv6Address {
    pub address: [u16; 8],
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
pub extern "C" fn registerOnDataCallback(cb: Option<OnDataCallback>) -> c_int
{
    unsafe{onDataCallback_ = cb;}
    return 0;
}

#[no_mangle]
pub extern "C" fn init(cb: Option<OnDataCallback>) -> c_int
{
    unsafe{onDataCallback_ = cb;}
    return 0;
}

#[no_mangle]
pub extern "C" fn smol_stack_tun_smol_stack_new<'a, 'b: 'a, 'c: 'a + 'b>(interface_name: &str) -> Box<TunSmolStack<'a, 'b, 'c>> {
    TunSmolStack::new(String::from(interface_name))
}

#[no_mangle]
pub extern "C" fn smol_stack_add_socket(tun_smol_stack: &mut TunSmolStack, socket_type: u8) -> Box<SocketHandle>  {
    match socket_type {
        0 => tun_smol_stack.add_socket(SocketType::TCP),
        1 => tun_smol_stack.add_socket(SocketType::UDP),
        _ => panic!("wrong type")
    }
}
pub extern "C" fn smol_stack_spin(tun_smol_stack: &mut TunSmolStack, socket_handle: &SocketHandle) {
    let timestamp = Instant::now();
    
    match tun_smol_stack.interface.as_mut().unwrap().poll(&mut tun_smol_stack.sockets, timestamp) {
        Ok(_) => {},
        Err(e) => {
            //debug!("poll error: {}",e);
        }
    }

    let mut socket = tun_smol_stack.sockets.get::<TcpSocket>(*socket_handle);
    let local_port = 49152 + rand::random::<u16>() % 16384;
    socket.connect((Ipv4Address::new(172, 217, 29, 14), 80), local_port).unwrap();
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