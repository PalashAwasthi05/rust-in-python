#[derive(Debug, Clone)]
struct Reading {
    sensor: &'static str,
    value: f64,
}

fn main() {
    let mut xs = vec![3.5, -1.25, 10.0, 0.0, 2.75, -7.5];
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("{:?}", xs);
    xs.sort_by(|a, b| b.partial_cmp(a).unwrap());
    println!("{:?}", xs);

    let mut ys = vec![2.0, f64::NAN, -0.0, 0.0, f64::INFINITY, -1.0, f64::NEG_INFINITY];
    ys.sort_by(|a, b| a.total_cmp(b));
    println!("{:?}", ys);
    ys.reverse();
    ys.sort_by(f64::total_cmp);
    println!("{:?}", ys);

    println!("1.0 < 2.0: {}", 1.0_f64 < 2.0);
    println!("nan < 1: {}", f64::NAN < 1.0);
    println!("nan > 1: {}", f64::NAN > 1.0);
    println!("nan == nan: {}", f64::NAN == f64::NAN);
    println!("partial nan: {:?}", f64::NAN.partial_cmp(&1.0));
    println!("partial: {:?}", 2.5_f64.partial_cmp(&2.5));
    println!("-0 == 0: {}", -0.0_f64 == 0.0);
    println!("total -0 vs 0: {:?}", (-0.0_f64).total_cmp(&0.0));
    println!("max: {}", 3.0_f64.max(f64::NAN));
    println!("min: {}", 3.0_f64.min(-2.0));

    let readings = vec![
        Reading { sensor: "a", value: 20.5 },
        Reading { sensor: "b", value: 19.0 },
        Reading { sensor: "c", value: 25.25 },
        Reading { sensor: "d", value: 25.25 },
        Reading { sensor: "e", value: 18.0 },
    ];
    let hottest = readings
        .iter()
        .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
        .unwrap();
    println!("hottest: {} {}", hottest.sensor, hottest.value);
    let coldest = readings.iter().min_by(|a, b| a.value.total_cmp(&b.value)).unwrap();
    println!("coldest: {} {}", coldest.sensor, coldest.value);
    let total: f64 = readings.iter().map(|r| r.value).sum();
    println!("mean: {:.3}", total / readings.len() as f64);
    let mut sorted = readings.clone();
    sorted.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap().then_with(|| a.sensor.cmp(b.sensor)));
    for r in &sorted {
        println!("{} {:.2}", r.sensor, r.value);
    }
    let fmax = readings.iter().map(|r| r.value).fold(f64::MIN, f64::max);
    println!("fold max: {}", fmax);
    let fmin = readings.iter().map(|r| r.value).fold(f64::INFINITY, |a, b| a.min(b));
    println!("fold min: {}", fmin);
    let fs: Vec<f32> = vec![1.5, 0.25, -3.0];
    let mut fs2 = fs.clone();
    fs2.sort_by(|a, b| a.total_cmp(b));
    println!("{:?}", fs2);
    println!("clamp: {}", 7.5_f64.clamp(0.0, 5.0));
}
