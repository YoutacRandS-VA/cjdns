//! Test Wrapper
//!
//! This is used with RustIface_test.c to verify that the Rust/C Iface bridge works as expected.

use crate::cffi::{self, Message};
use crate::external::interface::iface::{self, IfRecv, Iface, IfacePvt};
use anyhow::Result;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Default)]
struct TestWrapperPvt {
    incoming_count: usize,
    outgoing_count: usize,
}
impl Drop for TestWrapperPvt {
    fn drop(&mut self) {
        unsafe { cffi::RustIface_dropped() };
        println!("TestWrapperPvt dropped");
    }
}

pub struct TestWrapperInt {
    pvt: Arc<Mutex<TestWrapperPvt>>,
    ext_pvt: IfacePvt,
}
impl IfRecv for TestWrapperInt {
    fn recv(&self, m: &mut Message) -> Result<()> {
        {
            let mut pvt_l = self.pvt.lock().unwrap();
            pvt_l.outgoing_count += 1;
            unsafe { cffi::RustIface_gotOutgoing() };
            println!(
                "Received outgoing message, total out {} in {}",
                pvt_l.outgoing_count, pvt_l.incoming_count
            );
        }
        self.ext_pvt.send(m)
    }
}
impl Drop for TestWrapperInt {
    fn drop(&mut self) {
        unsafe { cffi::RustIface_dropped() };
        println!("TestWrapperInt dropped");
    }
}

pub struct TestWrapperExt {
    pvt: Arc<Mutex<TestWrapperPvt>>,
    int_pvt: IfacePvt,
}
impl IfRecv for TestWrapperExt {
    fn recv(&self, m: &mut Message) -> Result<()> {
        {
            let mut pvt_l = self.pvt.lock().unwrap();
            pvt_l.incoming_count += 1;
            unsafe { cffi::RustIface_gotIncoming() };
            println!(
                "Received incoming message, total out {} in {}",
                pvt_l.outgoing_count, pvt_l.incoming_count
            );
        }
        self.int_pvt.send(m)
    }
}
impl Drop for TestWrapperExt {
    fn drop(&mut self) {
        unsafe { cffi::RustIface_dropped() };
        println!("TestWrapperExt dropped");
    }
}

pub struct TestWrapper {
    pub int: Iface,
    pub ext: Iface,
}
impl Default for TestWrapper {
    fn default() -> TestWrapper {
        let pvt = Arc::new(Mutex::new(TestWrapperPvt::default()));
        let (mut int, int_pvt) = iface::new("TestWrapper int");
        let (mut ext, ext_pvt) = iface::new("TestWrapper ext");
        int.set_receiver(TestWrapperInt {
            pvt: pvt.clone(),
            ext_pvt,
        });
        ext.set_receiver(TestWrapperExt { pvt, int_pvt });
        TestWrapper { int, ext }
    }
}
