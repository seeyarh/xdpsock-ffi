use xsk_rs::{
    socket::{BindFlags, SocketConfig, SocketConfigBuilder, XdpFlags},
    umem::{UmemConfig, UmemConfigBuilder},
    xsk::Xsk2,
};

use libc::{c_char, size_t};
use std::ffi::CStr;
use std::slice;

#[no_mangle]
pub extern "C" fn xsk_new<'a>(ifname: *const c_char) -> *mut Xsk2<'a> {
    let ifname = unsafe {
        assert!(!ifname.is_null());
        CStr::from_ptr(ifname)
    };

    let ifname = ifname.to_str().unwrap();

    let umem_config = UmemConfigBuilder::new()
        .frame_count(8192)
        .comp_queue_size(4096)
        .fill_queue_size(4096)
        .build()
        .unwrap();

    let socket_config = SocketConfigBuilder::new()
        .tx_queue_size(4096)
        .rx_queue_size(4096)
        .bind_flags(BindFlags::XDP_COPY)
        .xdp_flags(XdpFlags::XDP_FLAGS_SKB_MODE)
        .build()
        .unwrap();

    let n_tx_frames = umem_config.frame_count() / 2;

    let mut xsk = Xsk2::new(&ifname, 0, umem_config, socket_config, n_tx_frames as usize);
    Box::into_raw(Box::new(xsk))
}

#[no_mangle]
pub extern "C" fn xsk_delete(xsk_ptr: *mut Xsk2) {
    let xsk = unsafe {
        assert!(!xsk_ptr.is_null());
        &mut *xsk_ptr
    };

    let tx_stats = xsk.shutdown_tx().expect("failed to shutdown tx");
    let rx_stats = xsk.shutdown_rx().expect("failed to shut down rx");
    eprintln!("tx_stats = {:?}", tx_stats);
    eprintln!("tx duration = {:?}", tx_stats.duration());
    eprintln!("tx pps = {:?}", tx_stats.pps());
    eprintln!("rx_stats = {:?}", rx_stats);
    unsafe {
        Box::from_raw(xsk_ptr);
    }
}

#[no_mangle]
pub extern "C" fn xsk_send(xsk_ptr: *mut Xsk2, pkt: *const u8, len: size_t) {
    let xsk = unsafe {
        assert!(!xsk_ptr.is_null());
        &mut *xsk_ptr
    };

    let pkt = unsafe {
        assert!(!pkt.is_null());
        slice::from_raw_parts(pkt, len as usize)
    };

    xsk.send(&pkt);
}

#[no_mangle]
pub extern "C" fn xsk_recv(xsk_ptr: *mut Xsk2, pkt: *mut u8, len: size_t) {
    let xsk = unsafe {
        assert!(!xsk_ptr.is_null());
        &mut *xsk_ptr
    };

    let mut pkt = unsafe {
        assert!(!pkt.is_null());
        slice::from_raw_parts_mut(pkt, len as usize)
    };

    let (recvd_pkt, len) = xsk.recv().expect("failed to recv");
    pkt[..len].clone_from_slice(&recvd_pkt[..len]);
}
