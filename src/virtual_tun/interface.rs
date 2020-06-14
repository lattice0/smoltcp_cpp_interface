use std::str::{self, FromStr};
use std::collections::BTreeMap;
use std::os::raw::{c_int};
use std::thread;
//use url::Url;
//use smoltcp::phy::wait as phy_wait;
use smoltcp::wire::{Ipv4Address, Ipv6Address, IpAddress, IpCidr};
use smoltcp::iface::{NeighborCache, InterfaceBuilder, Routes};
use smoltcp::socket::{SocketSet, TcpSocket, TcpSocketBuffer};
use smoltcp::time::Instant;
use super::smol_stack::{TunSmolStack, TunSmolStackBuilder};
//use smoltcp_openvpn_bridge::virtual_tun::VirtualTunInterface;

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
pub extern "C" fn smol_stack_add_ipv4_address(tun_smol_stack_builder: &mut TunSmolStackBuilder, cidr: CIpv4Cidr) {
    tun_smol_stack_builder.add_ipv4_address(cidr);
}

#[no_mangle]
pub extern "C" fn smol_stack_add_ipv6_address(tun_smol_stack_builder: &mut TunSmolStackBuilder, cidr: CIpv6Cidr) {
    tun_smol_stack_builder.add_ipv6_address(cidr);
}

#[no_mangle]
pub extern "C" fn smol_stack_add_default_v4_gateway(tun_smol_stack_builder: &mut TunSmolStackBuilder, address: CIpv4Address) {
    tun_smol_stack_builder.add_default_v4_gateway(address);
}

#[no_mangle]
pub extern "C" fn smol_stack_add_default_v6_gateway(tun_smol_stack_builder: &mut TunSmolStackBuilder, address: CIpv6Address) {
    tun_smol_stack_builder.add_default_v6_gateway(address);
}

#[no_mangle]
pub extern "C" fn smol_stack_finalize<'a, 'b: 'a, 'c: 'a + 'b>(tun_smol_stack_builder: &TunSmolStackBuilder<'a, 'b, 'c>) -> Box<TunSmolStack<'a, 'b, 'c>> {
    tun_smol_stack_builder.finalize()
}