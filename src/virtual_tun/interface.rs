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
    pub address: [u8; 4],
}

#[repr(C)]
pub struct CIpv6Address {
    pub address: [u16; 8],
}

#[repr(C)]
pub struct CIpv4Cidr {
    pub address: CIpv4Address,
    pub prefix: u32
}

#[repr(C)]
pub struct CIpv6Cidr {
    pub address: CIpv6Address,
    pub prefix: u64
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