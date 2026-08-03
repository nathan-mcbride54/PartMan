//! Reviewed Linux loop-device ioctl boundary (SAFE-009).
//!
//! This is the only module in the crate allowed to contain `unsafe`. Each
//! wrapper fixes one Linux UAPI opcode to its exact argument layout; no generic
//! ioctl or raw descriptor escapes this module. Structs and constants come from
//! `linux-raw-sys` 0.12.1 rather than handwritten ABI copies.

#![allow(unsafe_code)]

use core::ptr;
use std::os::fd::{AsFd, AsRawFd};

use linux_raw_sys::{ioctl as block, loop_device as raw};
use rustix::io::{Errno, Result};
use rustix::ioctl::{
    Getter, IntegerSetter, Ioctl, IoctlOutput, NoArg, Opcode, Setter, Updater, ioctl,
};

use crate::protocol::ConfigureRequest;

const OP_CONFIGURE: Opcode = raw::LOOP_CONFIGURE as Opcode;
const OP_GET_STATUS64: Opcode = raw::LOOP_GET_STATUS64 as Opcode;
const OP_CHANGE_FD: Opcode = raw::LOOP_CHANGE_FD as Opcode;
const OP_CLEAR_FD: Opcode = raw::LOOP_CLR_FD as Opcode;
const OP_CONTROL_GET_FREE: Opcode = raw::LOOP_CTL_GET_FREE as Opcode;
const OP_BLOCK_SIZE_GET: Opcode = block::BLKSSZGET as Opcode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Status {
    pub(super) backing_device: u64,
    pub(super) backing_inode: u64,
    pub(super) offset: u64,
    pub(super) size_limit: u64,
    pub(super) number: u32,
    pub(super) flags: u32,
}

struct GetFree;

unsafe impl Ioctl for GetFree {
    type Output = i32;

    const IS_MUTATING: bool = false;

    fn opcode(&self) -> Opcode {
        OP_CONTROL_GET_FREE
    }

    fn as_ptr(&mut self) -> *mut linux_raw_sys::ctypes::c_void {
        ptr::null_mut()
    }

    unsafe fn output_from_ptr(
        output: IoctlOutput,
        _pointer: *mut linux_raw_sys::ctypes::c_void,
    ) -> Result<Self::Output> {
        Ok(output)
    }
}

pub(super) fn control_get_free(control: impl AsFd) -> Result<i32> {
    // SAFETY: GetFree fixes LOOP_CTL_GET_FREE to its documented no-argument,
    // return-value ABI. The borrowed descriptor remains live for the call.
    unsafe { ioctl(control, GetFree) }
}

pub(super) fn configure(
    loop_device: impl AsFd,
    backing: impl AsFd,
    request: ConfigureRequest,
) -> Result<()> {
    let backing_fd = u32::try_from(backing.as_fd().as_raw_fd()).map_err(|_| Errno::BADF)?;
    let config = build_config(backing_fd, request);

    // SAFETY: LOOP_CONFIGURE expects a readable pointer to exactly one
    // `struct loop_config`. linux-raw-sys supplies that repr(C) layout; every
    // field and reserved byte is initialized, and both borrowed fds outlive the
    // call. The kernel copies the structure and takes its own backing reference.
    unsafe { ioctl(loop_device, Setter::<OP_CONFIGURE, _>::new(config)) }
}

fn build_config(backing_fd: u32, request: ConfigureRequest) -> raw::loop_config {
    raw::loop_config {
        fd: backing_fd,
        block_size: request.block_size,
        info: raw::loop_info64 {
            lo_offset: request.offset,
            lo_sizelimit: request.size_limit,
            lo_flags: request.flags,
            ..zero_info()
        },
        __reserved: [0; 8],
    }
}

pub(super) fn status(loop_device: impl AsFd) -> Result<Status> {
    let mut info = zero_info();
    // SAFETY: LOOP_GET_STATUS64 reads/writes exactly one `struct loop_info64`.
    // The value is fully initialized before the call, including all reserved,
    // name, and key bytes, so a kernel that leaves padding or a future field
    // untouched cannot cause userspace to materialize uninitialized memory.
    unsafe {
        ioctl(
            loop_device,
            Updater::<OP_GET_STATUS64, raw::loop_info64>::new(&mut info),
        )?;
    }
    Ok(decode_status(info))
}

fn decode_status(info: raw::loop_info64) -> Status {
    Status {
        backing_device: info.lo_device,
        backing_inode: info.lo_inode,
        offset: info.lo_offset,
        size_limit: info.lo_sizelimit,
        number: info.lo_number,
        flags: info.lo_flags,
    }
}

const fn zero_info() -> raw::loop_info64 {
    raw::loop_info64 {
        lo_device: 0,
        lo_inode: 0,
        lo_rdevice: 0,
        lo_offset: 0,
        lo_sizelimit: 0,
        lo_number: 0,
        lo_encrypt_type: 0,
        lo_encrypt_key_size: 0,
        lo_flags: 0,
        lo_file_name: [0; 64],
        lo_crypt_name: [0; 64],
        lo_encrypt_key: [0; 32],
        lo_init: [0; 2],
    }
}

pub(super) fn block_size(loop_device: impl AsFd) -> Result<i32> {
    // SAFETY: BLKSSZGET writes one C `int`; Rust i32 has that Linux UAPI width.
    unsafe { ioctl(loop_device, Getter::<OP_BLOCK_SIZE_GET, i32>::new()) }
}

pub(super) fn change_fd(loop_device: impl AsFd, backing: impl AsFd) -> Result<()> {
    let backing_fd = usize::try_from(backing.as_fd().as_raw_fd()).map_err(|_| Errno::BADF)?;
    // SAFETY: LOOP_CHANGE_FD takes a non-negative file descriptor as its
    // integer argument. The backing descriptor is borrowed and live for the
    // call; the kernel acquires its own reference on success.
    unsafe {
        ioctl(
            loop_device,
            IntegerSetter::<OP_CHANGE_FD>::new_usize(backing_fd),
        )
    }
}

pub(super) fn clear_fd(loop_device: impl AsFd) -> Result<()> {
    // SAFETY: LOOP_CLR_FD has no argument. The borrowed loop descriptor remains
    // live so status can be queried to confirm detach afterward.
    unsafe { ioctl(loop_device, NoArg::<OP_CLEAR_FD>::new()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{FLAG_AUTOCLEAR, FLAG_PARTSCAN, FLAG_READ_ONLY, REQUIRED_FLAGS};
    use core::mem::{offset_of, size_of};

    // Requirements: SAFE-009
    //   The generated Linux UAPI types have the exact layout passed across the FFI boundary.
    // Evidence: generated_uapi_layout_matches_linux_loop_header_contract
    #[cfg(target_os = "linux")]
    #[test]
    fn generated_uapi_layout_matches_linux_loop_header_contract() {
        assert_eq!(size_of::<raw::loop_info64>(), 232);
        assert_eq!(offset_of!(raw::loop_info64, lo_flags), 52);
        assert_eq!(offset_of!(raw::loop_info64, lo_file_name), 56);
        assert_eq!(size_of::<raw::loop_config>(), 304);
        assert_eq!(offset_of!(raw::loop_config, info), 8);
        assert_eq!(offset_of!(raw::loop_config, __reserved), 240);
    }

    // Requirements: SAFE-001, SAFE-007, SAFE-009
    //   Pure protocol flags equal the generated kernel constants exactly.
    // Evidence: pure_request_flags_equal_the_generated_uapi_constants
    #[cfg(target_os = "linux")]
    #[test]
    fn pure_request_flags_equal_the_generated_uapi_constants() {
        assert_eq!(FLAG_READ_ONLY, raw::LO_FLAGS_READ_ONLY as u32);
        assert_eq!(FLAG_AUTOCLEAR, raw::LO_FLAGS_AUTOCLEAR as u32);
        assert_eq!(FLAG_PARTSCAN, raw::LO_FLAGS_PARTSCAN as u32);
        assert_eq!(
            REQUIRED_FLAGS,
            raw::LO_FLAGS_READ_ONLY as u32
                | raw::LO_FLAGS_AUTOCLEAR as u32
                | raw::LO_FLAGS_PARTSCAN as u32
        );
    }

    // Requirements: SAFE-009
    //   Every confined wrapper is pinned to the generated Linux UAPI opcode.
    // Evidence: opcodes_equal_the_generated_uapi_constants
    #[cfg(target_os = "linux")]
    #[test]
    fn opcodes_equal_the_generated_uapi_constants() {
        // Independent UAPI values catch a generated-binding or import mix-up;
        // the comparisons below then prove the wrappers use those bindings.
        assert_eq!(OP_CLEAR_FD, 0x4c01 as Opcode);
        assert_eq!(OP_GET_STATUS64, 0x4c05 as Opcode);
        assert_eq!(OP_CHANGE_FD, 0x4c06 as Opcode);
        assert_eq!(OP_CONFIGURE, 0x4c0a as Opcode);
        assert_eq!(OP_CONTROL_GET_FREE, 0x4c82 as Opcode);
        assert_eq!(OP_BLOCK_SIZE_GET, rustix::ioctl::opcode::none(0x12, 104));
        assert_eq!(OP_CONFIGURE, raw::LOOP_CONFIGURE as Opcode);
        assert_eq!(OP_GET_STATUS64, raw::LOOP_GET_STATUS64 as Opcode);
        assert_eq!(OP_CHANGE_FD, raw::LOOP_CHANGE_FD as Opcode);
        assert_eq!(OP_CLEAR_FD, raw::LOOP_CLR_FD as Opcode);
        assert_eq!(OP_CONTROL_GET_FREE, raw::LOOP_CTL_GET_FREE as Opcode);
        assert_eq!(OP_BLOCK_SIZE_GET, block::BLKSSZGET as Opcode);
    }

    // Requirements: SAFE-009
    //   LOOP_CONFIGURE sends no uninitialized or undeclared name/key/reserved bytes.
    // Evidence: configure_info_starts_with_every_non_request_field_zero
    #[cfg(target_os = "linux")]
    #[test]
    fn configure_info_starts_with_every_non_request_field_zero() {
        let request = ConfigureRequest::READ_ONLY_ACCEPTANCE;
        let config = build_config(42, request);
        assert_eq!(config.fd, 42);
        assert_eq!(config.block_size, request.block_size);
        assert_eq!(config.info.lo_offset, request.offset);
        assert_eq!(config.info.lo_sizelimit, request.size_limit);
        assert_eq!(config.info.lo_flags, request.flags);
        assert_eq!(config.info.lo_device, 0);
        assert_eq!(config.info.lo_inode, 0);
        assert_eq!(config.info.lo_rdevice, 0);
        assert_eq!(config.info.lo_number, 0);
        assert_eq!(config.info.lo_encrypt_type, 0);
        assert_eq!(config.info.lo_encrypt_key_size, 0);
        assert_eq!(config.info.lo_file_name, [0; 64]);
        assert_eq!(config.info.lo_crypt_name, [0; 64]);
        assert_eq!(config.info.lo_encrypt_key, [0; 32]);
        assert_eq!(config.info.lo_init, [0; 2]);
        assert_eq!(config.__reserved, [0; 8]);
    }

    // Requirements: SAFE-005, SAFE-007, SAFE-009
    //   Every status field used by verification is decoded from its exact UAPI field.
    // Evidence: status_decoder_maps_every_verification_field
    #[cfg(target_os = "linux")]
    #[test]
    fn status_decoder_maps_every_verification_field() {
        let info = raw::loop_info64 {
            lo_device: 11,
            lo_inode: 22,
            lo_offset: 33,
            lo_sizelimit: 44,
            lo_number: 55,
            lo_flags: 66,
            ..zero_info()
        };
        assert_eq!(
            decode_status(info),
            Status {
                backing_device: 11,
                backing_inode: 22,
                offset: 33,
                size_limit: 44,
                number: 55,
                flags: 66,
            }
        );
    }
}
