#![allow(unsafe_code)]
#![allow(unused)]

use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use std::vec::Vec;
//use std::os::unix::io::{RawFd, AsRawFd};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use super::smol_stack::Blob;
use smoltcp::phy::{self, Device, DeviceCapabilities, Medium};
use smoltcp::time::Instant;
use smoltcp::{Error, Result};

use std::isize;
use std::ops::Deref;
use std::slice; //, DerefMut };


/// A virtual TUN interface.
//#[derive(Debug)]
#[derive(Clone)]
pub struct VirtualTunInterface {
    //lower:  Rc<RefCell<sys::VirtualTunInterfaceDesc>>,
    //put lower with transmit capabilities here? I think no lower is needed, only internally used
    //lower:
    mtu: usize,
    packets_from_inside: Arc<Mutex<VecDeque<Vec<u8>>>>,
    packets_from_outside: Arc<Mutex<VecDeque<Blob>>>,
}

impl<'a> VirtualTunInterface {
    /// Attaches to a TAP interface called `name`, or creates it if it does not exist.
    ///
    /// If `name` is a persistent interface configured with UID of the current user,
    /// no special privileges are needed. Otherwise, this requires superuser privileges
    /// or a corresponding capability set on the executable.
    pub fn new(
        _name: &str,
        packets_from_inside: Arc<Mutex<VecDeque<Vec<u8>>>>,
        packets_from_outside: Arc<Mutex<VecDeque<Blob>>>,
    ) -> Result<VirtualTunInterface> {
        /*
        //let mut lower = sys::VirtualTunInterfaceDesc::new(name)?;
        //lower.attach_interface()?;
        //todo: 1500 is the right size?
        let mtu = 1500; //= lower.interface_mtu()?;
                        //ip packet example
        let packet1: &[u8] = &[
            69, 0, 0, 72, 203, 203, 64, 0, 64, 17, 163, 146, 192, 168, 255, 18, 10, 139, 1, 1, 221,
            255, 0, 53, 0, 52, 174, 21, 221, 124, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 6, 99, 104, 97,
            116, 45, 48, 4, 99, 111, 114, 101, 10, 107, 101, 121, 98, 97, 115, 101, 97, 112, 105,
            3, 99, 111, 109, 0, 0, 28, 0, 1,
        ];

        let mut v1: VecDeque<CBuffer> = VecDeque::new();
        let v2: VecDeque<CBuffer> = VecDeque::new();
        let c1 = unsafe { CBuffer::from_owning(packet1.as_ptr(), packet1.len()) };
        v1.push_back(c1.unwrap());
        */
        let mtu = 1500; //??
        Ok(VirtualTunInterface {
            mtu: mtu,
            packets_from_outside: packets_from_outside.clone(),
            packets_from_inside: packets_from_inside.clone(),
        })
    }

    fn recv(&mut self, buffer: &mut [u8]) -> core::result::Result<usize, u32> {
        match self.packets_from_outside.lock().unwrap().pop_front() {
            Some(packet) => {
                /*
                let buffer_packet = packet.deref();
                for (dst, src) in buffer.iter_mut().zip(buffer_packet) {
                    *dst = *src
                }
                */
                buffer.copy_from_slice(packet.data.as_slice());
                Ok(packet.data.len())
            }
            None => Err(1),
        }
    }
}

impl<'d> Device<'d> for VirtualTunInterface {
    type RxToken = RxToken;
    type TxToken = TxToken;

    fn capabilities(&self) -> DeviceCapabilities {
        let mut d = DeviceCapabilities::default();
        d.max_transmission_unit = self.mtu;
        d
    }

    fn receive(&'d mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        let mut buffer = vec![0; self.mtu];
        match self.recv(&mut buffer[..]) {
            Ok(size) => {
                buffer.resize(size, 0);
                let rx = RxToken { buffer };
                let tx = TxToken {
                    lower: Rc::new(RefCell::new(self.clone())),
                };
                Some((rx, tx))
            }
            Err(err) if err == 1 => None,
            Err(err) => panic!("{}", err),
        }
    }

    fn transmit(&'d mut self) -> Option<Self::TxToken> {
        Some(TxToken {
            lower: Rc::new(RefCell::new(self.clone())),
        })
    }

    fn medium(&self) -> Medium {
        Medium::Ip
    }
}

#[doc(hidden)]
pub struct RxToken {
    buffer: Vec<u8>,
}

impl phy::RxToken for RxToken {
    fn consume<R, F>(mut self, _timestamp: Instant, f: F) -> Result<R>
    where
        F: FnOnce(&mut [u8]) -> Result<R>,
    {
        f(&mut self.buffer[..])
    }
}

#[doc(hidden)]
pub struct TxToken {
    lower: Rc<RefCell<VirtualTunInterface>>,
}

impl<'a> phy::TxToken for TxToken {
    fn consume<R, F>(self, _timestamp: Instant, len: usize, f: F) -> Result<R>
    where
        F: FnOnce(&mut [u8]) -> Result<R>,
    {
        let mut lower = self.lower.as_ref().borrow_mut();
        let mut buffer = vec![0; len];
        let result = f(&mut buffer);
        println!("should send NOW packet with size {}", len);
        //lower.send(&buffer[..]).unwrap();
        //TODO: unwrap here?
        //TODO: only if result ok
        //let p = unsafe { CBuffer::from_owning(buffer.as_ptr(), buffer.len()) };
        use std::borrow::BorrowMut;
        lower.packets_from_inside.lock().unwrap().push_back(buffer);
        result
    }
}
