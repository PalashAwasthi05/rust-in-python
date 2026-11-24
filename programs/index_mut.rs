use std::ops::{Index, IndexMut};
struct Grid { w: usize, cells: Vec<u8> }
impl Index<(usize, usize)> for Grid { type Output = u8; fn index(&self, (r, c): (usize, usize)) -> &u8 { &self.cells[r * self.w + c] } }
impl IndexMut<(usize, usize)> for Grid { fn index_mut(&mut self, (r, c): (usize, usize)) -> &mut u8 { &mut self.cells[r * self.w + c] } }
fn main() { let mut g = Grid { w: 3, cells: vec![0; 9] }; g[(1, 2)] = 7; println!("{}", g[(1, 2)]); }
