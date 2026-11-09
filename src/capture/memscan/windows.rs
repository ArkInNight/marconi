use super::{ChunkedScanner, CHUNK_SIZE};
use crate::error::AppError;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, STILL_ACTIVE};
use windows_sys::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows_sys::Win32::System::Memory::{
    VirtualQueryEx, MEMORY_BASIC_INFORMATION, MEM_COMMIT, MEM_IMAGE, MEM_MAPPED, MEM_PRIVATE,
    PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_WRITECOPY, PAGE_GUARD, PAGE_READONLY,
    PAGE_READWRITE, PAGE_WRITECOPY,
};
use windows_sys::Win32::System::Threading::{GetExitCodeProcess, OpenProcess};

const PROCESS_QUERY_INFORMATION: u32 = 0x0400;
const PROCESS_VM_READ: u32 = 0x0010;
const ERROR_ACCESS_DENIED: u32 = 5;

pub fn process_alive(pid: u32) -> bool {
    let handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid) };
    if handle.is_null() {
        return false;
    }
    let mut code: u32 = 0;
    let ok = unsafe { GetExitCodeProcess(handle, &mut code) } != 0;
    unsafe { CloseHandle(handle) };
    ok && code == STILL_ACTIVE as u32
}

fn readable(protect: u32) -> bool {
    let p = protect & !PAGE_GUARD;
    p & (PAGE_READONLY
        | PAGE_READWRITE
        | PAGE_WRITECOPY
        | PAGE_EXECUTE_READ
        | PAGE_EXECUTE_READWRITE
        | PAGE_EXECUTE_WRITECOPY)
        != 0
}

/// Region scan priority, mirroring the Linux backend:
/// - private (heap/stack/anonymous) mappings: always scanned (priority 0)
/// - writable mapped files: scanned only after a few passes (priority 1)
/// - read-only images and mapped files: skipped entirely
fn region_priority(info: &MEMORY_BASIC_INFORMATION) -> Option<u8> {
    let protect = info.Protect & !PAGE_GUARD;
    let writable = protect
        & (PAGE_READWRITE | PAGE_WRITECOPY | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY)
        != 0;
    match info.Type {
        MEM_PRIVATE => Some(0),
        MEM_MAPPED if writable => Some(1),
        _ => None,
    }
}

pub fn scan_process_memory(
    pid: u32,
    verbose: bool,
    include_file_mapped: bool,
) -> Result<Option<String>, AppError> {
    let handle = unsafe { OpenProcess(PROCESS_VM_READ | PROCESS_QUERY_INFORMATION, 0, pid) };
    if handle.is_null() {
        let err = std::io::Error::last_os_error();
        return Err(if err.raw_os_error() == Some(ERROR_ACCESS_DENIED as i32) {
            AppError::AccessDenied
        } else {
            AppError::from(format!("OpenProcess failed: {err}"))
        });
    }
    let result = scan_address_space(handle, verbose, include_file_mapped);
    unsafe { CloseHandle(handle) };
    result
}

fn scan_address_space(
    handle: HANDLE,
    verbose: bool,
    include_file_mapped: bool,
) -> Result<Option<String>, AppError> {
    let mut address: usize = 0;
    let mut denied = false;
    loop {
        let mut info = MEMORY_BASIC_INFORMATION::default();
        let len = unsafe {
            VirtualQueryEx(
                handle,
                address as *const std::ffi::c_void,
                &mut info,
                std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
            )
        };
        if len == 0 {
            break;
        }
        let region_start = address;
        let region_end = region_start + info.RegionSize;
        if info.State == MEM_COMMIT && readable(info.Protect) {
            if let Some(priority) = region_priority(&info) {
                if include_file_mapped || priority == 0 {
                    match scan_region(handle, region_start, region_end) {
                        Ok(Some(url)) => return Ok(Some(url)),
                        Ok(None) => {}
                        Err(AppError::AccessDenied) => denied = true,
                        Err(err) => {
                            if verbose {
                                println!(
                                    "[capture] skipped region {region_start:#x}-{region_end:#x}: {err}"
                                );
                            }
                        }
                    }
                }
            } else if verbose {
                println!("[capture] skipped image/mapped region {region_start:#x}-{region_end:#x}");
            }
        }
        if region_end <= address {
            break;
        }
        address = region_end;
    }
    if denied {
        return Err(AppError::AccessDenied);
    }
    Ok(None)
}

fn scan_region(handle: HANDLE, start: usize, end: usize) -> Result<Option<String>, AppError> {
    let mut buf = Vec::with_capacity(CHUNK_SIZE);
    let mut scanner = ChunkedScanner::new();
    let mut pos = start;
    while pos < end {
        let size = CHUNK_SIZE.min(end - pos);
        buf.clear();
        buf.resize(size, 0);
        let mut read: usize = 0;
        let ok = unsafe {
            ReadProcessMemory(
                handle,
                pos as *const std::ffi::c_void,
                buf.as_mut_ptr() as *mut std::ffi::c_void,
                size,
                &mut read,
            )
        };
        if ok == 0 {
            let err = std::io::Error::last_os_error();
            return Err(if err.raw_os_error() == Some(ERROR_ACCESS_DENIED as i32) {
                AppError::AccessDenied
            } else {
                AppError::from(format!("read failed at {pos:#x}: {err}"))
            });
        }
        if let Some(url) = scanner.push_and_scan(&buf[..read]) {
            return Ok(Some(url));
        }
        if read < size {
            break; // region shrank or became unreadable
        }
        pos += size;
    }
    Ok(None)
}
