// Imports renamed with `as`, impls through the alias, and Self constants in modules.
mod storage {
    #[derive(Debug, Default)]
    pub struct Buffer {
        data: Vec<u8>,
    }

    impl Buffer {
        pub const CAPACITY: usize = 4;

        pub fn push(&mut self, b: u8) -> bool {
            if self.data.len() >= Self::CAPACITY {
                return false;
            }
            self.data.push(b);
            true
        }

        pub fn len(&self) -> usize {
            self.data.len()
        }
    }
}

mod extras {
    use crate::storage::Buffer as Buf;

    impl Buf {
        pub fn fill(&mut self, b: u8) -> usize {
            let mut n = 0;
            while self.push(b) {
                n += 1;
            }
            n
        }

        pub fn remaining(&self) -> usize {
            Buf::CAPACITY - self.len()
        }
    }

    pub fn fresh() -> Buf {
        Buf::default()
    }
}

use storage::Buffer as Store;

fn main() {
    let mut b = extras::fresh();
    b.push(1);
    println!("{} {}", b.len(), b.remaining());
    println!("filled {}", b.fill(7));
    println!("{:?}", b);
    let s = Store::default();
    println!("{} {}", s.len(), Store::CAPACITY);
    println!("{}", storage::Buffer::CAPACITY + extras::fresh().remaining());
}
