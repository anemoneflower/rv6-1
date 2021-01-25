use crate::{
    kernel::Kernel,
    println,
    proc::{my_proc_data, my_proc_data_mut, myproc},
    vm::{UVAddr, VAddr},
};
use core::{mem, slice, str};
use cstr_core::CStr;

/// Fetch the usize at addr from the current process.
/// Returns Ok(fetched integer) on success, Err(()) on error.
pub unsafe fn fetchaddr(addr: UVAddr) -> Result<usize, ()> {
    let data = my_proc_data_mut();
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
    my_proc_data_mut().pagetable.copy_in_str(buf, addr)?;

    Ok(CStr::from_ptr(buf.as_ptr()))
}

fn argraw(n: usize) -> usize {
    // This is safe because we only read trapframe.
    let trapframe = unsafe { &*my_proc_data().trapframe };
    match n {
        0 => trapframe.a0,
        1 => trapframe.a1,
        2 => trapframe.a2,
        3 => trapframe.a3,
        4 => trapframe.a4,
        5 => trapframe.a5,
        _ => panic!("argraw"),
    }
}

/// Fetch the nth 32-bit system call argument.
pub fn argint(n: usize) -> Result<i32, ()> {
    Ok(argraw(n) as i32)
}

/// Retrieve an argument as a pointer.
/// Doesn't check for legality, since
/// copyin/copyout will do that.
pub fn argaddr(n: usize) -> Result<usize, ()> {
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
                    (*myproc()).pid(),
                    str::from_utf8(&(*myproc()).name).unwrap_or("???"),
                    num
                );
                Err(())
            }
        }
    }
}
