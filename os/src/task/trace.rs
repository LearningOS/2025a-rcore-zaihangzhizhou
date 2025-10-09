//! Implementation of [`TraceContext`]
use alloc::vec::Vec;

#[repr(C)]

/// trace context structure containing the ids and counts
pub struct TraceContext {
    ids:Vec<usize>,
    counts:Vec<usize>,
}

impl TraceContext {
    /// Create a new empty trace context
    pub fn new()-> Self {
        Self {
            ids:Vec::new(),
            counts:Vec::new()
        }
    }

    /// Update the count of syscall
    pub fn trace_syscall(&mut self,syscall_id:usize){
        if let Some(index)=self.ids.iter().position(|&id|id==syscall_id){
            self.counts[index]+=1;
        }else{
            self.ids.push(syscall_id);
            self.counts.push(1);
        }
    }

    ///Get the count of syscall
    pub fn get_count(&self,syscall_id:usize)->usize{
        if let Some(index)=self.ids.iter().position(|&id|id==syscall_id){
            return self.counts[index];
        }
        0
    }
}