//! File and filesystem-related syscalls
use crate::fs::{get_link_count, linkat, open_file, unlinkat, OpenFlags, Stat, StatMode};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_write", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_read", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        trace!("kernel: sys_read .. file.read");
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    trace!("kernel:pid[{}] sys_open", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    trace!("kernel:pid[{}] sys_close", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

/// YOUR JOB: Implement fstat.
pub fn sys_fstat(_fd: usize, _st: *mut Stat) -> isize {
    trace!(
        "kernel:pid[{}] sys_fstat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let token=current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access(); 
    if _fd>=inner.fd_table.len(){
        return -1;
    }
    if inner.fd_table[_fd].is_none(){
        return -1;
    }
    for (fd,entry) in inner.fd_table.iter().enumerate(){
        if fd==_fd{
            if let Some(file)=entry{
                if let Some(inode)=file.as_inode(){
                    let stat=Stat{
                        dev:0,
                        ino:inode.get_block_id() as u64,
                        mode:StatMode::FILE,
                        nlink:get_link_count(inode)as u32,
                        pad:[0;7],
                    };

                    let stat_size=core::mem::size_of::<Stat>();
                    let stat_bytes=unsafe{
                        core::slice::from_raw_parts(&stat as *const Stat as *const u8,stat_size)
                    };

                    let mut copied=0;
                    let buffers=translated_byte_buffer(token, _st as *const u8, stat_size);

                    for buffer in buffers{
                        let len=core::cmp::min(stat_size-copied,buffer.len());
                        if len==0{
                            break;
                        }
                        buffer[..len].copy_from_slice(&stat_bytes[copied..copied+len]);
                        copied+=len;
                    }
                }
            }
        }
    }
    0
}

/// YOUR JOB: Implement linkat.
pub fn sys_linkat(_old_name: *const u8, _new_name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_linkat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let old_name=translated_str(current_user_token(),_old_name);
    let new_name=translated_str(current_user_token(),_new_name);
    if old_name.as_str()==new_name.as_str(){
        -1
    }else{
        if let Some(_)=linkat(old_name.as_str(), new_name.as_str()){
            0
        }else{
            -1
        }
    }
}

/// YOUR JOB: Implement unlinkat.
pub fn sys_unlinkat(_name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_unlinkat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let name=translated_str(current_user_token(),_name);
    unlinkat(name.as_str())
}
