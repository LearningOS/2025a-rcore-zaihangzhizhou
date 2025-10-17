//! Mutex (spin-like and blocking(sleep))

use super::UPSafeCell;
use crate::task::{current_process, TaskControlBlock};
use crate::task::{block_current_and_run_next, suspend_current_and_run_next};
use crate::task::{current_task, wakeup_task};
use alloc::vec;
use alloc::{collections::VecDeque, sync::Arc};

/// Mutex trait
pub trait Mutex: Sync + Send {
    /// Try lock the mutex
    fn try_lock(&self, id_op:Option<usize>)->Option<isize>;
    /// Lock the mutex
    fn lock(&self, id_op:Option<usize>);
    /// Unlock the mutex
    fn unlock(&self, id_op:Option<usize>);
}

/// Spinlock Mutex struct
pub struct MutexSpin {
    locked: UPSafeCell<bool>,
}

impl MutexSpin {
    /// Create a new spinlock mutex
    pub fn new() -> Self {
        Self {
            locked: unsafe { UPSafeCell::new(false) },
        }
    }
}

impl Mutex for MutexSpin {
    /// Try lock the spinlock mutex
    fn try_lock(&self, id_op:Option<usize>)->Option<isize>{
        trace!("kernel: MutexSpin::try_lock");

        let id=id_op.unwrap();
        let locked = self.locked.exclusive_access();
        if *locked{
            let tid=current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
            let process = current_process();
            let process_inner = process.inner_exclusive_access();

            if process_inner.enable_deadlock_detect{
                let mut work=process_inner.mutex_available.clone();
                let mut need=process_inner.mutex_need.clone();
                let allocation=process_inner.mutex_allocation.clone();
                drop(process_inner);

                need[tid][id]+=1;
                let task_num=allocation.len();
                let mut finish=vec![false;task_num];
                let mut unfinish_num =work.len();
                
                loop{
                    let mut flag=false;
                    for i in  0..task_num{
                        if !finish[i]{
                            for j in 0..work.len(){
                                if need[i][j]>work[j]{
                                    break;
                                }
                                if j==work.len()-1{
                                    for k in 0..work.len(){
                                        work[k]+=allocation[i][k];
                                    }
                                    flag=true;
                                    finish[i]=true;
                                    unfinish_num-=1;
                                }
                            }
                            if flag{
                                break;
                            }
                        }
                    }
                    if !flag||unfinish_num==0{
                        break;
                    }
                }
                if unfinish_num!=0{
                    return Some(-0xdead);
                }
            }
        }
        None
    }

    /// Lock the spinlock mutex
    fn lock(&self, id_op:Option<usize>) {
        trace!("kernel: MutexSpin::lock");
        if id_op.is_none(){
            loop {
                let mut locked = self.locked.exclusive_access();
                if *locked {
                    drop(locked);
                    suspend_current_and_run_next();
                    continue;
                } else {
                    *locked = true;
                    return;
                }
            }
        }

        let id=id_op.unwrap();
        let tid=current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
        let process=current_process();
        let mut process_inner=process.inner_exclusive_access(); 
        process_inner.mutex_need[tid][id]+=1;
        drop(process_inner);

        loop {
            let mut locked = self.locked.exclusive_access();
            if *locked {
                drop(locked);
                suspend_current_and_run_next();
                continue;
            } else {
                let process=current_process();
                let mut process_inner=process.inner_exclusive_access();
                process_inner.mutex_allocation[tid][id]+=1;
                process_inner.mutex_need[tid][id]-=1;
                process_inner.mutex_available[id]-=1;

                *locked = true;
                return;
            }
        }
    }

    fn unlock(&self, id_op:Option<usize>) {
        trace!("kernel: MutexSpin::unlock");
        if id_op.is_none(){
            let mut locked = self.locked.exclusive_access();
            *locked = false;
            return;
        }

        let id=id_op.unwrap();
        let tid=current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
        let process=current_process();
        let mut process_inner=process.inner_exclusive_access();

        process_inner.mutex_allocation[tid][id]-=1;
        process_inner.mutex_available[id]+=1;
        drop(process_inner);

        let mut locked = self.locked.exclusive_access();
        *locked = false;
    }
}

/// Blocking Mutex struct
pub struct MutexBlocking {
    inner: UPSafeCell<MutexBlockingInner>,
}

pub struct MutexBlockingInner {
    locked: bool,
    wait_queue: VecDeque<Arc<TaskControlBlock>>,
}

impl MutexBlocking {
    /// Create a new blocking mutex
    pub fn new() -> Self {
        trace!("kernel: MutexBlocking::new");
        Self {
            inner: unsafe {
                UPSafeCell::new(MutexBlockingInner {
                    locked: false,
                    wait_queue: VecDeque::new(),
                })
            },
        }
    }
}

impl Mutex for MutexBlocking {
    /// try lock the blocking mutex
    fn try_lock(&self, id_op:Option<usize>)->Option<isize>{
        trace!("kernel: MutexBlocking::try_lock");
        let id=id_op.unwrap();
        let mutex_inner = self.inner.exclusive_access();
        if mutex_inner.locked{
            let tid=current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
            let process = current_process();
            let process_inner = process.inner_exclusive_access();

            if process_inner.enable_deadlock_detect{
                let mut work=process_inner.mutex_available.clone();
                let mut need=process_inner.mutex_need.clone();
                let allocation=process_inner.mutex_allocation.clone();
                drop(process_inner);

                need[tid][id]+=1;
                let task_num=allocation.len();
                let mut finish=vec![false;task_num];
                let mut unfinish_num =work.len();
                
                loop{
                    let mut flag=false;
                    for i in  0..task_num{
                        if !finish[i]{
                            for j in 0..work.len(){
                                if need[i][j]>work[j]{
                                    break;
                                }
                                if j==work.len()-1{
                                    for k in 0..work.len(){
                                        work[k]+=allocation[i][k];
                                    }
                                    flag=true;
                                    finish[i]=true;
                                    unfinish_num-=1;
                                }
                            }
                            if flag{
                                break;
                            }
                        }
                    }
                    if !flag||unfinish_num==0{
                        break;
                    }
                }
                if unfinish_num!=0{
                    return Some(-0xdead);
                }
            }
        }
        None
    }

    /// lock the blocking mutex
    fn lock(&self, id_op:Option<usize>) {
        trace!("kernel: MutexBlocking::lock");
        if id_op.is_none(){
            let mut mutex_inner = self.inner.exclusive_access();
            if mutex_inner.locked {
                mutex_inner.wait_queue.push_back(current_task().unwrap());
                drop(mutex_inner);
                block_current_and_run_next();
            } else {
                mutex_inner.locked = true;
            }
            return;
        }

        let id=id_op.unwrap();
        let tid=current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
        let process=current_process();
        let mut process_inner=process.inner_exclusive_access();

        let mut mutex_inner = self.inner.exclusive_access();
        if mutex_inner.locked {
            process_inner.mutex_need[tid][id]+=1;
            drop(process_inner);

            mutex_inner.wait_queue.push_back(current_task().unwrap());
            drop(mutex_inner);
            block_current_and_run_next();
        } else {
            process_inner.mutex_allocation[tid][id]+=1;
            process_inner.mutex_available[id]-=1;
            drop(process_inner);
            mutex_inner.locked = true;
        }
    }

    /// unlock the blocking mutex
    fn unlock(&self, id_op:Option<usize>) {
        trace!("kernel: MutexBlocking::unlock");
        if id_op.is_none(){
            let mut mutex_inner = self.inner.exclusive_access();
            assert!(mutex_inner.locked);
            if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
                wakeup_task(waking_task);
            } else {
                mutex_inner.locked = false;
            }
            return;
        }

        let id=id_op.unwrap();
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            let task_inner=waking_task.inner_exclusive_access();
            let tid=task_inner.res.as_ref().unwrap().tid;
            drop(task_inner);

            let process=current_process();
            let mut process_inner=process.inner_exclusive_access();
            process_inner.mutex_allocation[tid][id]+=1;
            process_inner.mutex_need[tid][id]-=1;
            drop(process_inner);
            wakeup_task(waking_task);
        } else {
            let process=current_process();
            let mut process_inner=process.inner_exclusive_access();
            process_inner.mutex_available[id]+=1;
            drop(process_inner);
            mutex_inner.locked = false;
        }
    }
}
