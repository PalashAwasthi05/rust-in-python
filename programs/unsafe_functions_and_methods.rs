struct S;
impl S { unsafe fn raw(&self) -> i32 { helper() } fn safe(&self) -> i32 { unsafe { self.raw() } } }
unsafe fn helper() -> i32 { 7 }
unsafe fn twice() -> i32 { helper() * 2 }
fn main() { let s = S; let f = || unsafe { helper() }; println!("{} {} {}", s.safe(), unsafe { twice() }, f()); }
