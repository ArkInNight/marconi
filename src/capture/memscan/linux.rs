use super::MemScanEngine;
use crate::error::AppError;
use std::io;
use std::path::Path;

const CHUNK_SIZE: usize = 8 * 1024 * 1024;
const OVERLAP: usize = 8192;

struct AttachGuard {
    pid: u32,
    attached: bool,
}

impl AttachGuard {
    fn attach(pid: u32) -> Result<Self, AppError> {
        // Detect a running process (rejects zombies).
        if !Path::new(&format!("/proc/{pid}")).exists() {
            return Err(AppError::NotRunning);
        }
        let ret = unsafe { libc::ptrace(libc::PTRACE_ATTACH, pid, 0, 0) };
        if ret != 0 {
            let err = io::Error::last_os_error();
            return Err(if err.raw_os_error() == Some(libc::EPERM) {
                AppError::AccessDenied
            } else {
                AppError::from(format!("ptrace attach failed: {err}"))
            });
        }
        let mut status: i32 = 0;
        unsafe {
            libc::waitpid(pid as libc::pid_t, &mut status, 0);
        }
        Ok(Self {
            pid,
            attached: true,
        })
    }
}

impl Drop for AttachGuard {
    fn drop(&mut self) {
        if self.attached {
            unsafe {
                libc::ptrace(libc::PTRACE_DETACH, self.pid, 0, 0);
            }
        }
    }
}

pub fn process_alive(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}

struct Region {
    priority: u8,
    start: u64,
    end: u64,
}

fn readable_regions(pid: u32) -> Result<Vec<Region>, AppError> {
    let maps = std::fs::read_to_string(format!("/proc/{pid}/maps"))
        .map_err(|e| AppError::from(format!("cannot read maps: {e}")))?;

    let mut regions: Vec<Region> = Vec::new();
    for line in maps.lines() {
        let mut fields = line.split_whitespace();
        let range = match fields.next() {
            Some(r) => r,
            None => continue,
        };
        let perms = match fields.next() {
            Some(p) => p,
            None => continue,
        };
        let path = fields.next().unwrap_or("");

        if !perms.contains('r') {
            continue;
        }
        let (start, end) = match range.split_once('-') {
            Some((a, b)) => (
                u64::from_str_radix(a, 16).ok(),
                u64::from_str_radix(b, 16).ok(),
            ),
            None => (None, None),
        };
        let (start, end) = match (start, end) {
            (Some(a), Some(b)) if b > a => (a, b),
            _ => continue,
        };
        if path.contains("[vvar]") || path.contains("[vdso]") {
            continue;
        }

        let anon = path.is_empty() || path.contains("[heap]") || path.contains("/dev/zero");
        let priority: u8 = if anon {
            0
        } else if perms.contains('w') {
            1
        } else {
            continue; // read-only file mappings (code, textures) are skipped
        };
        regions.push(Region {
            priority,
            start,
            end,
        });
    }

    regions.sort_by_key(|r| r.priority);
    Ok(regions)
}

fn read_mem(pid: u32, addr: u64, len: usize) -> io::Result<Vec<u8>> {
    let mut buf = vec![0u8; len];
    let local = libc::iovec {
        iov_base: buf.as_mut_ptr() as *mut libc::c_void,
        iov_len: len,
    };
    let remote = libc::iovec {
        iov_base: addr as *mut libc::c_void,
        iov_len: len,
    };
    let n = unsafe {
        libc::process_vm_readv(
            pid as libc::pid_t,
            &local as *const libc::iovec,
            1,
            &remote as *const libc::iovec,
            1,
            0,
        )
    };
    if n < 0 {
        return Err(io::Error::last_os_error());
    }
    buf.truncate(n as usize);
    Ok(buf)
}

fn scan_range(pid: u32, start: u64, end: u64) -> Result<Option<String>, AppError> {
    let mut carry: Vec<u8> = Vec::new();
    let mut pos = start;
    while pos < end {
        let size = CHUNK_SIZE.min((end - pos) as usize);
        let buf = read_mem(pid, pos, size)
            .map_err(|e| AppError::from(format!("read failed at {pos:#x}: {e}")))?;
        let mut combined = std::mem::take(&mut carry);
        combined.extend_from_slice(&buf);
        if let Some(url) = MemScanEngine::scan_buffer(&combined) {
            return Ok(Some(url));
        }
        let keep = OVERLAP.min(combined.len());
        carry = combined[combined.len() - keep..].to_vec();
        if buf.len() < size {
            break; // region shrank or became unreadable
        }
        pos += size as u64;
    }
    Ok(None)
}

pub fn scan_process_memory(pid: u32) -> Result<Option<String>, AppError> {
    let guard = AttachGuard::attach(pid)?;
    let regions = readable_regions(pid)?;
    for region in regions {
        if let Some(url) = scan_range(pid, region.start, region.end)? {
            return Ok(Some(url));
        }
    }
    drop(guard);
    Ok(None)
}
