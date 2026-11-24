// f64 math, constants and NaN handling.
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    fn len(self) -> f64 {
        self.x.hypot(self.y)
    }
    fn angle_deg(self) -> f64 {
        self.y.atan2(self.x).to_degrees()
    }
}

fn mean_std(xs: &[f64]) -> (f64, f64) {
    let n = xs.len() as f64;
    let mean = xs.iter().sum::<f64>() / n;
    let var = xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    (mean, var.sqrt())
}

fn main() {
    let v = Vec2 { x: 3.0, y: 4.0 };
    println!("len {} angle {:.4}", v.len(), v.angle_deg());
    println!("sqrt {} cbrt {} powi {} powf {:.6}", 2f64.sqrt(), 27f64.cbrt(), 1.5f64.powi(3), 2f64.powf(0.5));
    println!("floor {} ceil {} round {} trunc {} fract {:.2}", 2.7f64.floor(), 2.1f64.ceil(), -2.5f64.round(), -2.7f64.trunc(), 2.75f64.fract());
    println!("round half {} {} {}", 0.5f64.round(), 1.5f64.round(), 2.5f64.round());
    println!("abs {} signum {} {} copysign {}", (-3.5f64).abs(), (-0.0f64).signum(), 2f64.signum(), 3f64.copysign(-1.0));
    println!("to_radians {:.6} sin {:.6} cos {:.6} tan {:.6}", 180f64.to_radians(), (PI / 6.0).sin(), (PI / 3.0).cos(), (PI / 4.0).tan());
    println!("asin {:.6} acos {:.6} atan {:.6}", 1f64.asin(), 0f64.acos(), 1f64.atan());
    println!("exp {:.6} ln {:.6} log10 {} log2 {} log {:.6}", 1f64.exp(), std::f64::consts::E.ln(), 1000f64.log10(), 8f64.log2(), 9f64.log(3.0));
    println!("min {} max {} clamp {}", 1.5f64.min(-2.0), 1.5f64.max(f64::NAN), 7.3f64.clamp(0.0, 5.0));
    println!("mul_add {} recip {} rem {}", 2f64.mul_add(3.0, 1.0), 4f64.recip(), 7.5f64 % 2.0);
    println!("rem_euclid {} div_euclid {}", (-7.5f64).rem_euclid(2.0), (-7.5f64).div_euclid(2.0));
    println!("EPSILON {} MAX {:e} MIN {:e}", f64::EPSILON, f64::MAX, f64::MIN);
    println!("approx eq {}", ((0.1 + 0.2) - 0.3f64).abs() < f64::EPSILON);
    let nan = f64::NAN;
    println!("nan==nan {} is_nan {} partial_cmp {:?}", nan == nan, nan.is_nan(), nan.partial_cmp(&1.0));
    println!("is_finite {} {} is_infinite {}", 1f64.is_finite(), f64::INFINITY.is_finite(), (-1f64 / 0.0).is_infinite());
    println!("0/0 nan {} sqrt(-1) nan {}", (0.0f64 / 0.0).is_nan(), (-1f64).sqrt().is_nan());
    println!("sign bits {} {}", (-0.0f64).is_sign_negative(), 0.0f64.is_sign_positive());
    let mut xs = vec![3.5, -1.0, f64::NAN, 2.25, 0.0];
    xs.retain(|x| !x.is_nan());
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("sorted {:?}", xs);
    let mut ys = vec![2.0, -0.0, 0.0, f64::INFINITY, -1.0];
    ys.sort_by(f64::total_cmp);
    println!("total_cmp {:?}", ys);
    let (mean, sd) = mean_std(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
    println!("mean {} sd {}", mean, sd);
    println!("f32 {} {} {}", 0.1f32 + 0.2f32, 1f32 / 3.0, (2.0f32).sqrt());
    println!("f32 consts {} {}", f32::MAX, f32::EPSILON);
    println!("to_bits {:#x} from_bits {}", 1.0f64.to_bits(), f64::from_bits(0x4000000000000000));
    println!("int-ish {} {} {}", 1e15, 1e16, 123456789.0f64 * 10.0);
    println!("small {} {} {}", 1e-5, 0.000123, 5e-324);
    println!("debug {:?} {:?} {:?} {:?}", 1e16, 1e-5, 0.3f64, 100.0f32);
    let celsius = [-40.0, 0.0, 37.0, 100.0];
    for c in celsius.iter() {
        println!("{:>6.1}C = {:>6.1}F", c, c * 9.0 / 5.0 + 32.0);
    }
    let area = PI * 2.0f64.powi(2);
    println!("area {:.3} {}", area, area as i64);
    println!("max of floats {:?}", celsius.iter().cloned().fold(f64::MIN, f64::max));
    println!("sum f32 {}", [0.1f32; 10].iter().sum::<f32>());
    println!("lerp {}", 10.0 + (20.0 - 10.0) * 0.25);
    println!("consts {:.5} {:.5} {:.5}", std::f64::consts::SQRT_2, std::f64::consts::LN_2, std::f64::consts::TAU);
    println!("sinh {:.4} cosh {:.4} tanh {:.4}", 1f64.sinh(), 1f64.cosh(), 1f64.tanh());
    println!("exp2 {} ln_1p {:.6} exp_m1 {:.6}", 10f64.exp2(), 0.5f64.ln_1p(), 0.5f64.exp_m1());
    println!("i32 from f {} {}", (2.5f64).round() as i32, (-0.4f64).round());
}
