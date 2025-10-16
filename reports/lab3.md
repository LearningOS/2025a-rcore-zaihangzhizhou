# 实现的功能
重写sys_get_time、sys_mmap、sys_munmap。
实现sys_spawn、sys_set_priority系统调用与stride算法。sys_spawn在创建进程时从传入的elf中构造地址空间而不是复制父进程地址空间。sys_set_priority设置TCB中新增参数prio。stride算法重写fetch函数，遍历找出stride最小的进程移除并返回，更新stride。

# 问答题
## 题1
### 1
实际轮到p2执行。  
使用8bit无符号整数保存stride，且pass均为10，初始p1.stride=255,p2.stride=250,p2执行一个时间片后stride溢出，p2.stride=4,stride算法取stride最小的任务执行，执行p2。  

### 2
当前需要证明STRIDE_MAX – STRIDE_MIN <= BigStride / 2。  
记当前stride_max的任务为A,stride_min的任务为B,则A在执行上一次时间片前stride_A<= stride_B=STRIDE_MIN,stride_A+BigStride/prio_A=STRIDE_MAX。STRIDE_MAX-STRIDE_MIN=stride_A+BigStride/prio_A-STRIDE_MIN<= stride_A+BigStride/prio_A-stride_A=BigStride/prio_A,又已知进程优先级 >= 2，prio_A>=2,所以STRIDE_MAX-STRIDE_MIN<=BigStride/prio_A<=BigStride / 2,得证。  

### 3
fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    let v1=self.0;
    let v2=other.0;
    if (v1-v2).abs()<=BigStride/2{
        v1.partial_cmp(&v2)
    }else{
        v2.partial_cmp(&v1)
    }
}

# 荣誉准则
1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

无

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

无

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。