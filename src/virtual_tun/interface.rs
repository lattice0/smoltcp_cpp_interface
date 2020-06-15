use std::str::{self};
use std::os::raw::{c_int};

use super::smol_stack::{TunSmolStackBuilder};

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
pub extern "C" fn smol_stack_tun_smol_stack_builder_new<'a, 'b: 'a, 'c: 'a + 'b>(interface_name: &str) -> Box<TunSmolStackBuilder<'a, 'b, 'c>> {
    TunSmolStackBuilder::new(String::from(interface_name))
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
pub extern "C" fn smol_stack_finalize<'a, 'b: 'a, 'c: 'a + 'b>(tun_smol_stack_builder: &TunSmolStackBuilder<'a, 'b, 'c>) -> u8 {
    tun_smol_stack_builder.finalize()
}