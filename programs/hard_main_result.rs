use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct ValidationError { field: String, reason: String }

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.reason)
    }
}
impl Error for ValidationError {}

#[derive(Debug)]
struct Employee { name: String, age: u8, salary: u32, dept: String }

fn parse_line(line: &str) -> Result<Employee, Box<dyn Error>> {
    let fields: Vec<&str> = line.split(',').map(str::trim).collect();
    if fields.len() != 4 {
        return Err(Box::new(ValidationError { field: "line".into(), reason: format!("expected 4 fields, got {}", fields.len()) }));
    }
    let age: u8 = fields[1].parse()?;
    let salary: u32 = fields[2].parse()?;
    if fields[0].is_empty() {
        return Err("name must not be empty".into());
    }
    Ok(Employee { name: fields[0].to_string(), age, salary, dept: fields[3].to_string() })
}

fn load(data: &str) -> (Vec<Employee>, Vec<String>) {
    let mut ok = Vec::new();
    let mut errs = Vec::new();
    for (i, line) in data.lines().enumerate() {
        match parse_line(line) {
            Ok(e) => ok.push(e),
            Err(e) => errs.push(format!("line {}: {}", i + 1, e)),
        }
    }
    (ok, errs)
}

fn payroll(emps: &[Employee]) -> Result<BTreeMap<&str, u32>, Box<dyn Error>> {
    let mut by_dept = BTreeMap::new();
    for e in emps {
        let total: &mut u32 = by_dept.entry(e.dept.as_str()).or_insert(0);
        *total = total.checked_add(e.salary).ok_or("payroll overflow")?;
    }
    Ok(by_dept)
}

fn main() -> Result<(), Box<dyn Error>> {
    let data = "Ann, 34, 72000, eng\nBob, 300, 50000, ops\nCy, 28, 64000, eng\n, 40, 1, ops\nDee, 45, 81000\nEd, 51, 59000, ops\nFay, 22, lots, eng";
    let (emps, errs) = load(data);
    println!("loaded {} employees", emps.len());
    for e in &errs { println!("  skipped {}", e); }
    let totals = payroll(&emps)?;
    for (dept, total) in &totals { println!("{:<4} {:>7}", dept, total); }
    let oldest = emps.iter().max_by_key(|e| e.age).ok_or("no employees")?;
    println!("oldest: {} ({})", oldest.name, oldest.age);
    let n: i32 = "  17 ".trim().parse()?;
    let hex = i64::from_str_radix("ff", 16)?;
    println!("parsed {} and {}", n, hex);
    let huge = vec![Employee { name: "X".into(), age: 1, salary: u32::MAX, dept: "z".into() }, Employee { name: "Y".into(), age: 1, salary: 1, dept: "z".into() }];
    match payroll(&huge) {
        Ok(_) => println!("unexpected ok"),
        Err(e) => println!("payroll error: {}", e),
    }
    let r: Result<u8, _> = "256".parse::<u8>();
    if let Err(e) = &r { println!("u8 parse: {}", e); }
    println!("done");
    Ok(())
}
