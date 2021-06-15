use xdpsock::{
    socket::{BindFlags, SocketConfigBuilder, XdpFlags},
    umem::UmemConfigBuilder,
    xsk::Xsk2,
};

use libc::{c_char, size_t};
use std::ffi::CStr;
use std::slice;

#[no_mangle]
pub unsafe extern "C" fn xsk_new(ifname: *const c_char) -> *mut Xsk2 {
    let ifname = {
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

    let n_tx_batch_size = 1024;

    let xsk = Xsk2::new(
        &ifname,
        0,
        umem_config,
        socket_config,
        n_tx_frames as usize,
        n_tx_batch_size,
    )
    .expect("failed to build xsk");
    Box::into_raw(Box::new(xsk))
}

#[no_mangle]
pub unsafe extern "C" fn xsk_delete(xsk_ptr: *mut Xsk2) {
    let _xsk = {
        assert!(!xsk_ptr.is_null());
        &mut *xsk_ptr
    };

    Box::from_raw(xsk_ptr);
}

#[no_mangle]
pub unsafe extern "C" fn xsk_send(xsk_ptr: *mut Xsk2, pkt: *const u8, len: size_t) {
    let xsk = {
        assert!(!xsk_ptr.is_null());
        &mut *xsk_ptr
    };

    let pkt = {
        assert!(!pkt.is_null());
        slice::from_raw_parts(pkt, len as usize)
    };

    xsk.tx.send(&pkt).expect("failed to send pkt");
}

#[no_mangle]
pub unsafe extern "C" fn xsk_recv(xsk_ptr: *mut Xsk2, pkt: *mut u8, len: size_t) -> u16 {
    let xsk = {
        assert!(!xsk_ptr.is_null());
        &mut *xsk_ptr
    };

    let pkt = {
        assert!(!pkt.is_null());
        slice::from_raw_parts_mut(pkt, len as usize)
    };

    xsk.rx.recv(pkt) as u16
}
