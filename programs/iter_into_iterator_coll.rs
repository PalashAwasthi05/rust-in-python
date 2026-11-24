// IntoIterator for an owned collection, a shared reference and a mutable reference.
#[derive(Debug)]
struct Shelf {
    books: Vec<String>,
}

impl Shelf {
    fn new() -> Self {
        Shelf { books: Vec::new() }
    }
    fn add(&mut self, title: &str) {
        self.books.push(title.to_string());
    }
    fn iter(&self) -> std::slice::Iter<'_, String> {
        self.books.iter()
    }
}

impl IntoIterator for Shelf {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;
    fn into_iter(self) -> Self::IntoIter {
        self.books.into_iter()
    }
}

impl<'a> IntoIterator for &'a Shelf {
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;
    fn into_iter(self) -> Self::IntoIter {
        self.books.iter()
    }
}

impl<'a> IntoIterator for &'a mut Shelf {
    type Item = &'a mut String;
    type IntoIter = std::slice::IterMut<'a, String>;
    fn into_iter(self) -> Self::IntoIter {
        self.books.iter_mut()
    }
}

struct Grid {
    width: usize,
    cells: Vec<u8>,
}

struct Rows<'a> {
    grid: &'a Grid,
    row: usize,
}

impl<'a> Iterator for Rows<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<&'a [u8]> {
        let start = self.row * self.grid.width;
        if start >= self.grid.cells.len() {
            return None;
        }
        self.row += 1;
        Some(&self.grid.cells[start..start + self.grid.width])
    }
}

impl<'a> IntoIterator for &'a Grid {
    type Item = &'a [u8];
    type IntoIter = Rows<'a>;
    fn into_iter(self) -> Rows<'a> {
        Rows { grid: self, row: 0 }
    }
}

fn total_len<'a, I>(items: I) -> usize
where
    I: IntoIterator<Item = &'a String>,
{
    items.into_iter().map(|s| s.len()).sum()
}

fn main() {
    let mut shelf = Shelf::new();
    shelf.add("dune");
    shelf.add("emma");
    shelf.add("ulysses");

    for book in &shelf {
        println!("reading {}", book);
    }

    for book in &mut shelf {
        book.make_ascii_uppercase();
        book.push('!');
    }
    println!("{:?}", shelf);

    println!("total chars: {}", total_len(&shelf));
    println!("via iter(): {}", shelf.iter().filter(|b| b.len() > 5).count());

    let lengths: Vec<usize> = (&shelf).into_iter().map(|b| b.len()).collect();
    println!("lengths {:?}", lengths);

    let grid = Grid { width: 3, cells: vec![1, 2, 3, 4, 5, 6, 7, 8, 9] };
    for (i, row) in (&grid).into_iter().enumerate() {
        println!("row {}: {:?} sum {}", i, row, row.iter().map(|&c| c as u32).sum::<u32>());
    }
    let mut diag = 0u32;
    for (i, row) in grid.into_iter().enumerate() {
        diag += row[i] as u32;
    }
    println!("diag {}", diag);

    let mut owned = Vec::new();
    for book in shelf {
        owned.push(book.to_lowercase());
    }
    println!("owned {:?}", owned);
}
