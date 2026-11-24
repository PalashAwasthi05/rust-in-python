// CSV-style parsing into records, grouping and aggregate statistics.
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Dept {
    Eng,
    Sales,
    Ops,
}

impl FromStr for Dept {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "eng" | "engineering" => Ok(Dept::Eng),
            "sales" => Ok(Dept::Sales),
            "ops" => Ok(Dept::Ops),
            other => Err(format!("unknown dept {:?}", other)),
        }
    }
}

#[derive(Debug, Clone)]
struct Employee {
    name: String,
    dept: Dept,
    salary: f64,
    years: Option<u8>,
}

impl fmt::Display for Employee {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:<6} {:?}/{:>9.2}", self.name, self.dept, self.salary)?;
        if let Some(y) = self.years {
            write!(f, " ({}y)", y)?;
        }
        Ok(())
    }
}

fn parse_line(line: &str) -> Result<Employee, String> {
    let fields: Vec<&str> = line.split(',').map(str::trim).collect();
    let [name, dept, salary, years] = fields[..] else {
        return Err(format!("expected 4 fields, got {}", fields.len()));
    };
    Ok(Employee {
        name: name.to_string(),
        dept: dept.parse()?,
        salary: salary.parse().map_err(|_| format!("bad salary {:?}", salary))?,
        years: if years.is_empty() { None } else { Some(years.parse().map_err(|e| format!("{}", e))?) },
    })
}

fn median(values: &mut Vec<f64>) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = values.len() / 2;
    Some(if values.len() % 2 == 0 { (values[mid - 1] + values[mid]) / 2.0 } else { values[mid] })
}

fn main() {
    let data = "name,dept,salary,years
ann, eng, 120000.5, 7
bob, Sales, 65000,
cy, ENG, 98000, 3
dee, ops, 72000.25, 12
eve, hr, 50000, 1
fay, sales, abc, 2
gus, ops, 81000, 300
hal, eng, 105000
ida, eng, 131000, 9";
    let (good, bad): (Vec<_>, Vec<_>) = data.lines().skip(1).map(parse_line).partition(Result::is_ok);
    let staff: Vec<Employee> = good.into_iter().map(Result::unwrap).collect();
    for err in bad.into_iter().filter_map(Result::err) {
        println!("skip: {}", err);
    }
    for e in &staff {
        println!("{}", e);
    }
    let mut by_dept: BTreeMap<Dept, Vec<&Employee>> = BTreeMap::new();
    for e in &staff {
        by_dept.entry(e.dept).or_default().push(e);
    }
    for (dept, people) in &by_dept {
        let total: f64 = people.iter().map(|e| e.salary).sum();
        let mut salaries: Vec<f64> = people.iter().map(|e| e.salary).collect();
        let top = people.iter().max_by(|a, b| a.salary.partial_cmp(&b.salary).unwrap()).map(|e| e.name.as_str());
        println!(
            "{:<6?} n={} total={:>10.2} avg={:>9.2} median={:?} top={}",
            dept,
            people.len(),
            total,
            total / people.len() as f64,
            median(&mut salaries),
            top.unwrap_or("-")
        );
    }
    let seniority: HashMap<bool, usize> = staff.iter().fold(HashMap::new(), |mut m, e| {
        *m.entry(e.years.map_or(false, |y| y >= 5)).or_insert(0) += 1;
        m
    });
    println!("senior {:?} junior {:?}", seniority.get(&true), seniority.get(&false));
    let avg_years = {
        let known: Vec<u32> = staff.iter().filter_map(|e| e.years).map(u32::from).collect();
        known.iter().sum::<u32>() as f64 / known.len() as f64
    };
    println!("avg years {:.3}", avg_years);
    let raise: Vec<(String, f64)> = staff.iter().map(|e| (e.name.to_uppercase(), (e.salary * 1.035 * 100.0).round() / 100.0)).collect();
    println!("{:?}", &raise[..2]);
    println!("{:?}", "Eng".parse::<Dept>().and_then(|d| "x".parse::<Dept>().map(|e| (d, e))));
    let names = staff.iter().map(|e| e.name.clone()).collect::<Vec<_>>().join(", ");
    println!("{}", names);
}
