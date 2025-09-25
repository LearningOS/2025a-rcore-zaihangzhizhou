//! Implementation of [`TraceContext`]

#[derive(Copy, Clone)]
#[repr(C)]

/// trace context structure containing the ids and counts
pub struct TraceContext {
    size:usize,
    ids:[usize;16],
    counts:[usize;16],
    
}

impl TraceContext {
    /// Create a new empty trace context
    pub fn init() -> Self {
        Self {
            size:0,
            ids:[0;16],
            counts:[0;16],
        }
    }

    /// Update the count of syscall
    pub fn trace_syscall(&mut self,syscall_id:usize){
        for i in 0..self.size{
            if self.ids[i]==syscall_id{
                self.counts[i]+=1;
                return;
            }
        }
        self.ids[self.size]=syscall_id;
        self.counts[self.size]=1;
        self.size+=1;
    }

    ///Get the count of syscall
    pub fn get_count(&self,syscall_id:usize)->usize{
        for i in 0..self.size{
            if self.ids[i]==syscall_id{
                return self.counts[i];
            }
        }
        0
    }
}
