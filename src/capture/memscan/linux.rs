use super::{ChunkedScanner, CHUNK_SIZE};
use crate::error::AppError;
use std::io;
use std::path::Path;

pub fn process_alive(pid: u32) -> bool {
    let path = format!("/proc/{pid}");
    if !Path::new(&path).exists() {
        return false;
    }
    // A zombie still has a /proc entry; treat it as dead so we don't hang.
    match std::fs::read_to_string(format!("{path}/stat")) {
        Ok(stat) => stat
            .rsplit(')')
            .next()
            .is_none_or(|rest| rest.split_whitespace().next() != Some("Z")),
        Err(_) => true,
    }
}

struct Region {
    priority: u8,
    start: u64,
    end: u64,
}

fn readable_regions(pid: u32) -> Result<Vec<Region>, AppError> {
    let maps = std::fs::read_to_string(format!("/proc/{pid}/maps")).map_err(|e| {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            AppError::AccessDenied
        } else {
            AppError::from(format!("cannot read maps: {e}"))
        }
    })?;

    let mut regions: Vec<Region> = Vec::new();
    for line in maps.lines() {
        let mut fields = line.split_whitespace();
        let Some(range) = fields.next() else {
            continue;
        };
        let Some(perms) = fields.next() else {
            continue;
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

/// Merge adjacent regions with the same priority so strings spanning region
/// boundaries are still picked up by the chunked carry scan.
fn merge_regions(regions: Vec<Region>) -> Vec<Region> {
    let mut merged: Vec<Region> = Vec::with_capacity(regions.len());
    for region in regions {
        if let Some(last) = merged.last_mut() {
            if last.priority == region.priority && last.end == region.start {
                last.end = region.end;
                continue;
            }
        }
        merged.push(region);
    }
    merged
}

fn is_access_denied(e: &io::Error) -> bool {
    matches!(e.raw_os_error(), Some(code) if code == libc::EPERM || code == libc::EACCES)
}

fn read_mem(pid: u32, addr: u64, buf: &mut [u8]) -> io::Result<usize> {
    let local = libc::iovec {
        iov_base: buf.as_mut_ptr().cast::<libc::c_void>(),
        iov_len: buf.len(),
    };
    let remote = libc::iovec {
        iov_base: addr as *mut libc::c_void,
        iov_len: buf.len(),
    };
    let n = unsafe {
        libc::process_vm_readv(
            pid.cast_signed(),
            &raw const local,
            1,
            &raw const remote,
            1,
            0,
        )
    };
    if n < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(n.cast_unsigned())
}

fn scan_range(pid: u32, start: u64, end: u64) -> Result<Option<String>, AppError> {
    let mut buf = Vec::with_capacity(CHUNK_SIZE);
    let mut scanner = ChunkedScanner::new();
    let mut pos = start;
    while pos < end {
        let remaining = usize::try_from(end - pos).unwrap_or(CHUNK_SIZE);
        let size = CHUNK_SIZE.min(remaining);
        buf.clear();
        buf.resize(size, 0);
        let n = read_mem(pid, pos, &mut buf).map_err(|e| {
            if is_access_denied(&e) {
                AppError::AccessDenied
            } else {
                AppError::from(format!("read failed at {pos:#x}: {e}"))
            }
        })?;
        if let Some(url) = scanner.push_and_scan(&buf[..n]) {
            return Ok(Some(url));
        }
        if n < size {
            break; // region shrank or became unreadable
        }
        pos += size as u64;
    }
    Ok(None)
}

pub fn scan_process_memory(
    pid: u32,
    verbose: bool,
    include_file_mapped: bool,
) -> Result<Option<String>, AppError> {
    let regions = readable_regions(pid)?;

    // Check permission immediately via dummy read on the first region (if any)
    if let Some(first) = regions.first() {
        let mut probe = [0u8; 1];
        if let Err(e) = read_mem(pid, first.start, &mut probe) {
            if is_access_denied(&e) {
                return Err(AppError::AccessDenied);
            }
        }
    }

    let mut denied = false;
    for region in merge_regions(regions) {
        if !include_file_mapped && region.priority > 0 {
            continue;
        }
        match scan_range(pid, region.start, region.end) {
            Ok(Some(url)) => return Ok(Some(url)),
            Ok(None) => {}
            Err(AppError::AccessDenied) => denied = true,
            Err(err) => {
                // Regions can be unmapped or change protection between reading
                // /proc/<pid>/maps and reading the memory (GC, JIT, allocator).
                // Skip the failing region instead of aborting the whole scan.
                if verbose {
                    println!(
                        "[capture] skipped region {:#x}-{:#x}: {err}",
                        region.start, region.end
                    );
                }
            }
        }
    }
    if denied {
        return Err(AppError::AccessDenied);
    }
    Ok(None)
}
