use call_dispatch_macro::call_dispatch;

struct Syscall;

const SYS_READ: u32 = 1;

#[call_dispatch]
impl Syscall {
    #[dispatcher]
    fn syscall(&mut self, num: u32, args: [usize; 6]) -> Option<i32> {
        panic!("code generated by macro")
    }

    #[call]
    fn sys_read(&mut self, fd: i32, buf: *mut u8, len: usize) -> i32 {
        println!("sys_read: fd={:?}, buf=({:?}; {:?})", fd, buf, len);
        1
    }
}

fn main() {
    let mut syscall = Syscall;
    assert_eq!(syscall.syscall(0, [0; 6]), None);
    assert_eq!(syscall.syscall(1, [0; 6]), Some(1));
}