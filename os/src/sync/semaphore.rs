//! Semaphore

use crate::sync::UPSafeCell;
use crate::task::{block_current_and_run_next, current_process, current_task, wakeup_task, TaskControlBlock};
use alloc::vec;
use alloc::{collections::VecDeque, sync::Arc};

/// semaphore structure
pub struct Semaphore {
    /// semaphore inner
    pub inner: UPSafeCell<SemaphoreInner>,
}

pub struct SemaphoreInner {
    pub count: isize,
    pub wait_queue: VecDeque<Arc<TaskControlBlock>>,
}

impl Semaphore {
    /// Create a new semaphore
    pub fn new(res_count: usize) -> Self {
        trace!("kernel: Semaphore::new");
        Self {
            inner: unsafe {
                UPSafeCell::new(SemaphoreInner {
                    count: res_count as isize,
                    wait_queue: VecDeque::new(),
                })
            },
        }
    }

    /// up operation of semaphore
    pub fn up(&self ,id:usize) {
        trace!("kernel: Semaphore::up");
        let mut inner = self.inner.exclusive_access();
        inner.count += 1;
        if inner.count <= 0 {
            if let Some(task) = inner.wait_queue.pop_front() {
                let task_inner=task.inner_exclusive_access();
                let tid=task_inner.res.as_ref().unwrap().tid;
                drop(task_inner);

                let process=current_process();
                let mut process_inner=process.inner_exclusive_access();
                process_inner.semaphore_allocation[tid][id]+=1;
                process_inner.semaphore_need[tid][id]-=1;
                drop(process_inner);

                wakeup_task(task);
            }
        }else{
            let process=current_process();
            let mut process_inner=process.inner_exclusive_access();
            process_inner.semaphore_available[id]+=1;
            drop(process_inner);
        }
    }

    /// try down operation of semaphore
    pub fn try_down(&self, id:usize)->Option<isize>{
        trace!("kernel: Semaphore::try_down");
        let inner = self.inner.exclusive_access();
        if inner.count <= 0 {
            let tid=current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
            let process = current_process();
            let process_inner = process.inner_exclusive_access();

            if process_inner.enable_deadlock_detect{
                let mut work=process_inner.semaphore_available.clone();
                let mut need=process_inner.semaphore_need.clone();
                let allocation=process_inner.semaphore_allocation.clone();
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

    /// down operation of semaphore
    pub fn down(&self, id:usize) {
        trace!("kernel: Semaphore::down");
        let tid=current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
        let process=current_process();
        let mut process_inner=process.inner_exclusive_access();

        let mut inner = self.inner.exclusive_access();
        inner.count -= 1;
        if inner.count < 0 {
            process_inner.semaphore_need[tid][id]+=1;
            drop(process_inner);

            inner.wait_queue.push_back(current_task().unwrap());
            drop(inner);
            block_current_and_run_next();
        }else{
            process_inner.semaphore_allocation[tid][id]+=1;
            process_inner.semaphore_available[id]-=1;
            drop(process_inner);
        }
    }
}
