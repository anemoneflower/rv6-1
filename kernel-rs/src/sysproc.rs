use crate::{
    kernel::{kernel, Kernel},
    poweroff,
    proc::{myproc, ExecutingProc},
    syscall::{argaddr, argint},
    vm::{UVAddr, VAddr},
};

impl Kernel {
    /// Terminate the current process; status reported to wait(). No return.
    pub unsafe fn sys_exit(&self, proc: &ExecutingProc) -> Result<usize, ()> {
        let n = unsafe { argint(0) }?;
        unsafe { self.procs.exit_current(n, proc) };
    }

    /// Return the current process’s PID.
    pub unsafe fn sys_getpid(&self) -> Result<usize, ()> {
        Ok(unsafe { (*myproc()).pid() } as _)
    }

    /// Create a process.
    /// Returns Ok(child’s PID) on success, Err(()) on error.
    pub unsafe fn sys_fork(&self, proc: &ExecutingProc) -> Result<usize, ()> {
        Ok(unsafe { self.procs.fork(proc) }? as _)
    }

    /// Wait for a child to exit.
    /// Returns Ok(child’s PID) on success, Err(()) on error.
    pub unsafe fn sys_wait(&self) -> Result<usize, ()> {
        let p = unsafe { argaddr(0) }?;
        Ok(unsafe { self.procs.wait(UVAddr::new(p)) }? as _)
    }

    /// Grow process’s memory by n bytes.
    /// Returns Ok(start of new memory) on success, Err(()) on error.
    pub unsafe fn sys_sbrk(&self) -> Result<usize, ()> {
        let n = unsafe { argint(0) }?;
        let p = unsafe { kernel().myexproc() };
        let data = p.deref_mut_data();
        data.memory.resize(n)
    }

    /// Pause for n clock ticks.
    /// Returns Ok(0) on success, Err(()) on error.
    pub unsafe fn sys_sleep(&self) -> Result<usize, ()> {
        let n = unsafe { argint(0) }?;
        let mut ticks = self.ticks.lock();
        let ticks0 = *ticks;
        while ticks.wrapping_sub(ticks0) < n as u32 {
            if unsafe { kernel().myexproc().killed() } {
                return Err(());
            }
            ticks.sleep();
        }
        Ok(0)
    }

    /// Terminate process PID.
    /// Returns Ok(0) on success, Err(()) on error.
    pub unsafe fn sys_kill(&self) -> Result<usize, ()> {
        let pid = unsafe { argint(0) }?;
        self.procs.kill(pid)?;
        Ok(0)
    }

    /// Return how many clock tick interrupts have occurred
    /// since start.
    pub fn sys_uptime(&self) -> Result<usize, ()> {
        Ok(*self.ticks.lock() as usize)
    }

    /// Shutdowns this machine, discarding all unsaved data. No return.
    pub unsafe fn sys_poweroff(&self) -> Result<usize, ()> {
        let exitcode = unsafe { argint(0) }?;
        poweroff::machine_poweroff(exitcode as _);
    }
}
