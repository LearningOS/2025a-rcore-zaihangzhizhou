//! Process management syscalls
use crate::{ 
    config::PAGE_SIZE, 
    mm::{read_one_byte, translated_byte_buffer, write_one_byte, MapPermission, VPNRange, VirtAddr}, 
    task::{change_program_brk, create_map_area, current_user_token, exit_current_and_run_next, suspend_current_and_run_next, syscall_count, translated_pte, unmap_vpn_range}, 
    timer::get_time_us
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");

    let us=get_time_us();
    let timeval=TimeVal{
        sec:us/1_000_000,
        usec:us%1_000_000
    };
    let timeval_size=core::mem::size_of::<TimeVal>();
    let buffers=translated_byte_buffer(current_user_token(),_ts as *const u8,timeval_size);

    let timeval_copy=unsafe{
        core::slice::from_raw_parts(&timeval as *const TimeVal as *const u8,timeval_size)
    };

    let mut index=0;
    for buffer in buffers{
        let copy_len=core::cmp::min(timeval_size-index,buffer.len());
        if copy_len>0{
            buffer[..copy_len].copy_from_slice(&timeval_copy[index..index+copy_len]);
            index+=copy_len;
        }else{
            break;
        }
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    match _trace_request{
        0=>{
            match read_one_byte(current_user_token(),_id){
                Some(res)=>res as isize,
                None=>-1
            }
        }
        1=>write_one_byte(current_user_token(),_id,_data),
        2=>syscall_count(_id) as isize,
        _=>-1
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if _start%PAGE_SIZE!=0||_port&!0x7!=0||_port&0x7==0{
        return -1;
    }
    let start_vpn=VirtAddr::from(_start).floor();
    let end_vpn=VirtAddr::from(_start+_len).ceil();
    let vpn_range=VPNRange::new(start_vpn,end_vpn);
    for vpn in vpn_range{
        if let Some(pte)=translated_pte(vpn){
            if pte.is_valid(){
                return -1;
            }
        }
    }
    let mut permission=MapPermission::U;
    if _port&1<<0!=0{
        permission|=MapPermission::R;
    }
    if _port&1<<1!=0{
        permission|=MapPermission::W;
    }
    if _port&1<<2!=0{
        permission|=MapPermission::X;
    }
    create_map_area(start_vpn.into(),end_vpn.into(),permission);
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if _start%PAGE_SIZE!=0{
        return -1;
    }
    let start_vpn=VirtAddr::from(_start).floor();
    let end_vpn=VirtAddr::from(_start+_len).ceil();
    let vpn_range=VPNRange::new(start_vpn,end_vpn);
    for vpn in vpn_range{
        if let Some(pte)=translated_pte(vpn){
            if !pte.is_valid(){
                return -1;
            }
        }else{
            return -1;
        }
    }
    unmap_vpn_range(vpn_range);
    0
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
