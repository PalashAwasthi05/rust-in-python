#[derive(Clone, Copy)]
struct Point { x: i32, y: i32 }

#[derive(Clone, Debug)]
struct Book { author: &'static str, title: &'static str, year: u32 }

fn eat_box_i32(boxed_i32: Box<i32>) {
    println!("destroying box that contains {}", boxed_i32);
}

fn borrow_i32(borrowed_i32: &i32) {
    println!("this int is: {}", borrowed_i32);
}

fn borrow_book(book: &Book) {
    println!("I immutably borrowed {} - {} edition", book.title, book.year);
}

fn new_edition(book: &mut Book) {
    book.year = 2014;
    println!("I mutably borrowed {} - {} edition", book.title, book.year);
}

fn longest_title(books: &[Book]) -> &Book {
    books.iter().max_by_key(|b| b.title.len()).unwrap()
}

fn push_all(dst: &mut Vec<i32>, src: &[i32]) {
    for &x in src { dst.push(x); }
}

fn main() {
    let boxed_i32 = Box::new(5_i32);
    let stacked_i32 = 6_i32;
    borrow_i32(&boxed_i32);
    borrow_i32(&stacked_i32);
    {
        let _ref_to_i32: &i32 = &boxed_i32;
        borrow_i32(_ref_to_i32);
    }
    eat_box_i32(boxed_i32);

    let immutabook = Book { author: "Douglas Hofstadter", title: "Gödel, Escher, Bach", year: 1979 };
    let mut mutabook = immutabook.clone();
    borrow_book(&immutabook);
    borrow_book(&mutabook);
    new_edition(&mut mutabook);
    println!("by {}", mutabook.author);

    // aliasing: many shared borrows or one mutable borrow
    let mut point = Point { x: 0, y: 0 };
    let borrowed_point = &point;
    let another_borrow = &point;
    println!("point has coordinates: ({}, {})", borrowed_point.x, another_borrow.y);
    let mutable_borrow = &mut point;
    mutable_borrow.x = 5;
    mutable_borrow.y = 2;
    println!("point has coordinates: ({}, {})", mutable_borrow.x, mutable_borrow.y);
    let new_borrowed_point = &point;
    println!("point now has coordinates: ({}, {})", new_borrowed_point.x, new_borrowed_point.y);

    // the ref pattern
    let c = 'Q';
    let ref ref_c1 = c;
    let ref_c2 = &c;
    println!("ref_c1 equals ref_c2: {}", *ref_c1 == *ref_c2);
    let p = Point { x: 0, y: 0 };
    let _copy_of_x = { let Point { x: ref ref_to_x, y: _ } = p; *ref_to_x };
    let mut mutable_point = p;
    {
        let Point { x: _, y: ref mut mut_ref_to_y } = mutable_point;
        *mut_ref_to_y = 1;
    }
    println!("point is ({}, {}), mutable_point is ({}, {})", p.x, p.y, mutable_point.x, mutable_point.y);
    let mut mutable_tuple = (Box::new(5u32), 3u32);
    {
        let (_, ref mut last) = mutable_tuple;
        *last = 2u32;
    }
    println!("tuple is {:?}", mutable_tuple);
    let maybe_name = Some(String::from("Alice"));
    if let Some(ref n) = maybe_name {
        println!("hello {}", n);
    }
    println!("still own {:?}", maybe_name);

    let shelf = vec![immutabook, mutabook, Book { author: "K&R", title: "C", year: 1978 }];
    println!("longest title: {}", longest_title(&shelf).title);
    let mut out = vec![0];
    push_all(&mut out, &[1, 2, 3]);
    let first = &out[0];
    println!("pushed {:?}, first {}", out, first);
    let mut counts = [0u32; 3];
    for (i, slot) in counts.iter_mut().enumerate() { *slot = (i as u32 + 1) * 10; }
    let total: u32 = counts.iter().sum();
    println!("counts {:?} total {}", counts, total);
}
