#![allow(unused)]
#![feature(register_tool)]
#![register_tool(rr)]
#![feature(custom_inner_attributes)]

fn main() {
    let mut a;
    let b;
    unsafe {
        a = EvenInt::new(0);
        b = EvenInt::new(2);
    }

    a.add_even(&b);
    assert!(a.get() % 2 == 0);
}

#[rr::refined_by("x" : "Z")]
#[rr::invariant("Zeven x")]
struct EvenInt {
    #[rr::field("x")]
    num: i32,
}

impl EvenInt {
    /// Create a new even integer.
    pub unsafe fn new(x: i32) -> Self {
        Self {num: x}
    }

    /// Internal function. Unsafely add 1, making the integer odd.
    unsafe fn add(&mut self) {
        self.num += 1;
    }

    /// Get the even integer
    pub fn get(&self) -> i32 {
        self.num
    }

    /// Add another even integer.
    pub fn add_even(&mut self, other: &Self) {
        self.num += other.num;
    }

    /// Add 2 to an even integer.
    pub fn add_two(&mut self) {
        self.num;

        unsafe {
            self.add();
            self.add();
        }
    }
}
