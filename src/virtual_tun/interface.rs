#[macro_use]
extern crate log;
extern crate env_logger;
extern crate getopts;
extern crate rand;
extern crate url;
extern crate smoltcp;

mod utils;

use std::str::{self, FromStr};
use std::collections::BTreeMap;
use std::os::raw::{c_int};
use std::thread;
use url::Url;
//use smoltcp::phy::wait as phy_wait;
use smoltcp::wire::{Ipv4Address, Ipv6Address, IpAddress, IpCidr};
use smoltcp::iface::{NeighborCache, TunInterfaceBuilder, Routes};
use smoltcp::socket::{SocketSet, TcpSocket, TcpSocketBuffer};
use smoltcp::time::Instant;
use smoltcp_openvpn_bridge::virtual_tun::VirtualTunInterface;

type OnDataCallback = unsafe extern "C" fn(data: *mut u8, len: usize) -> c_int;

static mut onDataCallback_: Option<OnDataCallback> = None;

#[repr(C)]
pub struct CIpv4Address {
    pub address: *mut u8
    pub cidr: *mut u8
}

#[repr(C)]
pub struct CIpv6Address {
    pub address: *mut u8
    pub cidr: *mut u8
}

#[repr(C)]
pub struct CIpv4Cidr {
    pub address: *mut CIpv4Address
    pub cidr: *mut u8
}

#[repr(C)]
pub struct CIpv6Cidr {
    pub address: *mut CIpv6Address
    pub cidr: *mut u8
}

#[repr(C)]
pub struct VPNParameters {
    pub ipv4_addresses: *mut CIpv4Cidr,
    pub ipv4_addresses_size: c_int,
    pub ipv6_addresses: *mut CIpv4Cidr,
    pub ipv6_addresses_size: c_int,
    pub default_ipv4_gateway: *mut CIpv4Address,
    pub default_ipv6_gateway: *mut CIpv6Address,
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