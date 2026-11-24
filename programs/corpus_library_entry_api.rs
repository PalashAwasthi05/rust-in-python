use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Book {
    title: String,
    author: String,
    copies: u32,
    loaned: u32,
}

#[derive(Default)]
struct Library {
    books: HashMap<String, Book>,
    loans: HashMap<String, Vec<String>>,
}

impl Library {
    fn add(&mut self, isbn: &str, title: &str, author: &str) {
        self.books.entry(isbn.to_string())
            .and_modify(|b| b.copies += 1)
            .or_insert_with(|| Book { title: title.into(), author: author.into(), copies: 1, loaned: 0 });
    }

    fn checkout(&mut self, member: &str, isbn: &str) -> Result<(), String> {
        let book = self.books.get_mut(isbn).ok_or(format!("no such isbn {}", isbn))?;
        if book.loaned >= book.copies {
            return Err(format!("'{}' unavailable", book.title));
        }
        book.loaned += 1;
        self.loans.entry(member.to_string()).or_default().push(isbn.to_string());
        Ok(())
    }

    fn return_book(&mut self, member: &str, isbn: &str) -> bool {
        if let Some(list) = self.loans.get_mut(member) {
            if let Some(pos) = list.iter().position(|i| i == isbn) {
                list.remove(pos);
                if list.is_empty() { self.loans.remove(member); }
                if let Some(b) = self.books.get_mut(isbn) { b.loaned -= 1; }
                return true;
            }
        }
        false
    }

    fn report(&self) {
        let mut keys: Vec<&String> = self.books.keys().collect();
        keys.sort();
        for k in keys {
            let b = &self.books[k];
            println!("  {} {:<22} {:<10} {}/{}", k, b.title, b.author, b.copies - b.loaned, b.copies);
        }
        let mut members: Vec<(&String, &Vec<String>)> = self.loans.iter().collect();
        members.sort();
        for (m, l) in members { println!("  {} has {:?}", m, l); }
    }
}

fn main() {
    let mut lib = Library::default();
    lib.add("111", "Dune", "Herbert");
    lib.add("222", "Neuromancer", "Gibson");
    lib.add("111", "Dune", "Herbert");
    lib.add("333", "The Left Hand of Dark", "Le Guin");
    let attempts = [("ana", "111"), ("ben", "111"), ("cy", "111"), ("ana", "222"), ("ben", "999"), ("cy", "333")];
    for (who, isbn) in attempts {
        match lib.checkout(who, isbn) {
            Ok(()) => println!("{} borrowed {}", who, isbn),
            Err(e) => println!("{} failed: {}", who, e),
        }
    }
    lib.report();
    println!("ana returns 111: {}", lib.return_book("ana", "111"));
    println!("ana returns 111 again: {}", lib.return_book("ana", "111"));
    println!("cy returns 333: {}", lib.return_book("cy", "333"));
    lib.report();
    let mut by_author: HashMap<&str, u32> = HashMap::new();
    for b in lib.books.values() { *by_author.entry(b.author.as_str()).or_insert(0) += b.copies; }
    let mut v: Vec<_> = by_author.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    println!("copies by author: {:?}", v);
}
