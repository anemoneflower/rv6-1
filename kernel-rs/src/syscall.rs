use crate::{
    kernel::Kernel,
    println,
    proc::myproc,
    vm::{UVAddr, VAddr},
};
use core::{mem, slice, str};
use cstr_core::CStr;

/// Fetch the usize at addr from the current process.
/// Returns Ok(fetched integer) on success, Err(()) on error.
pub unsafe fn fetchaddr(addr: UVAddr) -> Result<usize, ()> {
    let p = myproc();
    let data = (*p).deref_mut_procdata();
    let mut ip = 0;
    if addr.into_usize() >= data.sz
        || addr.into_usize().wrapping_add(mem::size_of::<usize>()) > data.sz
    {
        return Err(());
    }
    data.pagetable.copy_in(
        slice::from_raw_parts_mut(&mut ip as *mut usize as *mut u8, mem::size_of::<usize>()),
        addr,
    )?;
    Ok(ip)
}

/// Fetch the nul-terminated string at addr from the current process.
/// Returns reference to the string in the buffer.
pub unsafe fn fetchstr(addr: UVAddr, buf: &mut [u8]) -> Result<&CStr, ()> {
    let p = myproc();
    ((*p).deref_mut_procdata())
        .pagetable
        .copy_in_str(buf, addr)?;

    Ok(CStr::from_ptr(buf.as_ptr()))
}

unsafe fn argraw(n: usize) -> usize {
    let p = myproc();
    let data = (*p).deref_mut_procdata();
    match n {
        0 => (*data.trapframe).a0,
        1 => (*data.trapframe).a1,
        2 => (*data.trapframe).a2,
        3 => (*data.trapframe).a3,
        4 => (*data.trapframe).a4,
        5 => (*data.trapframe).a5,
        _ => panic!("argraw"),
    }
}

/// Fetch the nth 32-bit system call argument.
pub unsafe fn argint(n: usize) -> Result<i32, ()> {
    Ok(argraw(n) as i32)
}

/// Retrieve an argument as a pointer.
/// Doesn't check for legality, since
/// copyin/copyout will do that.
pub unsafe fn argaddr(n: usize) -> Result<usize, ()> {
    Ok(argraw(n))
}

/// Fetch the nth word-sized system call argument as a null-terminated string.
/// Copies into buf, at most max.
/// Returns reference to the string in the buffer.
pub unsafe fn argstr(n: usize, buf: &mut [u8]) -> Result<&CStr, ()> {
    let addr = argaddr(n)?;
    fetchstr(UVAddr::new(addr), buf)
}

impl Kernel {
    pub unsafe fn syscall(&'static self, num: i32) -> Result<usize, ()> {
        let p = myproc();

        match num {
            1 => self.sys_fork(),
            2 => self.sys_exit(),
            3 => self.sys_wait(),
            4 => self.sys_pipe(),
            5 => self.sys_read(),
            6 => self.sys_kill(),
            7 => self.sys_exec(),
            8 => self.sys_fstat(),
            9 => self.sys_chdir(),
            10 => self.sys_dup(),
            11 => self.sys_getpid(),
            12 => self.sys_sbrk(),
            13 => self.sys_sleep(),
            14 => self.sys_uptime(),
            15 => self.sys_open(),
            16 => self.sys_write(),
            17 => self.sys_mknod(),
            18 => self.sys_unlink(),
            19 => self.sys_link(),
            20 => self.sys_mkdir(),
            21 => self.sys_close(),
            22 => self.sys_poweroff(),
            _ => {
                println!(
                    "{} {}: unknown sys call {}",
                    (*p).pid(),
                    str::from_utf8(&(*p).name).unwrap_or("???"),
                    num
                );
                Err(())
            }
        }
    }
}
