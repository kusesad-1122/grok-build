// Android 移植实现:rustix 在 android 上未导出 pidfd_open/PidfdFlags,
// 这里直接用 libc 的 pidfd_open 系统调用 + poll 等待子进程退出,语义与 linux.rs 一致。
use std::io::{Error, ErrorKind, Result};
use std::os::fd::{FromRawFd, OwnedFd};
use std::time::Duration;

pub type WaitHandle = OwnedFd;

pub fn open(pid: i32) -> Result<WaitHandle> {
    if pid < 0 {
        return Err(Error::new(ErrorKind::InvalidInput, format!("invalid PID {pid}")));
    }
    // SYS_pidfd_open 在 Android(内核 5.3+)可用。
    let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, pid as libc::pid_t, 0) };
    if fd < 0 {
        return Err(Error::last_os_error());
    }
    // SAFETY: fd 为内核刚返回的、我们独占的有效文件描述符。
    Ok(unsafe { OwnedFd::from_raw_fd(fd as i32) })
}

pub fn wait(pidfd: &mut WaitHandle, timeout: Option<Duration>) -> Result<Option<()>> {
    use std::os::fd::AsRawFd;
    let millis: libc::c_int = match timeout {
        Some(dur) => dur.as_millis().min(libc::c_int::MAX as u128) as libc::c_int,
        None => -1, // 无限等待
    };
    let mut fds = [libc::pollfd {
        fd: pidfd.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    }];
    let ret = unsafe { libc::poll(fds.as_mut_ptr(), 1, millis) };
    if ret < 0 {
        return Err(Error::last_os_error());
    }
    if ret == 0 {
        // 超时。
        return Ok(None);
    }
    Ok(Some(()))
}
