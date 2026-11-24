use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct ProductId(u32);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Sku(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Location(char, u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Grade { A, B, C }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
struct Cents(u64);

impl std::ops::Add for Cents {
    type Output = Cents;
    fn add(self, o: Cents) -> Cents { Cents(self.0 + o.0) }
}

fn main() {
    let stock = vec![
        (ProductId(7), Sku("BOLT-M6".into()), Location('B', 2), Grade::A, 3, Cents(25)),
        (ProductId(3), Sku("NUT-M6".into()), Location('A', 1), Grade::B, 10, Cents(10)),
        (ProductId(7), Sku("BOLT-M6".into()), Location('A', 4), Grade::A, 5, Cents(25)),
        (ProductId(9), Sku("WASHER".into()), Location('B', 2), Grade::C, 40, Cents(4)),
        (ProductId(3), Sku("NUT-M6".into()), Location('C', 1), Grade::B, 2, Cents(10)),
    ];

    let mut qty_by_product: BTreeMap<ProductId, u32> = BTreeMap::new();
    let mut where_stored: HashMap<Sku, BTreeSet<Location>> = HashMap::new();
    let mut per_location: BTreeMap<Location, Vec<ProductId>> = BTreeMap::new();
    let mut value_by_grade: BTreeMap<Grade, Cents> = BTreeMap::new();
    let mut seen: HashSet<(ProductId, Grade)> = HashSet::new();

    for (id, sku, loc, grade, qty, price) in &stock {
        *qty_by_product.entry(*id).or_insert(0) += qty;
        where_stored.entry(sku.clone()).or_default().insert(*loc);
        per_location.entry(*loc).or_default().push(*id);
        let v = value_by_grade.entry(*grade).or_default();
        *v = *v + Cents(price.0 * *qty as u64);
        seen.insert((*id, *grade));
    }

    println!("qty by product: {:?}", qty_by_product);
    let mut skus: Vec<_> = where_stored.iter().collect();
    skus.sort();
    for (sku, locs) in skus {
        println!("{:?} stored at {:?}", sku, locs);
    }
    for (loc, ids) in &per_location {
        println!("{}{}: {:?}", loc.0, loc.1, ids);
    }
    println!("value by grade: {:?}", value_by_grade);
    let mut combos: Vec<_> = seen.into_iter().collect();
    combos.sort();
    println!("distinct (product, grade): {:?}", combos);

    let a = Location('A', 9);
    let b = Location('B', 1);
    println!("A9 < B1: {}, max: {:?}, eq: {}", a < b, a.max(b), a == Location('A', 9));
    println!("Grade ordering: {:?}", { let mut g = vec![Grade::C, Grade::A, Grade::B]; g.sort(); g });
    let total = value_by_grade.values().fold(Cents::default(), |acc, &c| acc + c);
    println!("total value: {:?} = ${}.{:02}", total, total.0 / 100, total.0 % 100);
    let cloned = Sku("X".to_string());
    println!("clone eq: {}, cmp: {:?}", cloned.clone() == cloned, Sku("A".into()).cmp(&Sku("B".into())));
}
