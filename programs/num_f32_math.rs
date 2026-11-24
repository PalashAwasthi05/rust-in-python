#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Self { Vec2 { x, y } }
    fn length(self) -> f32 { (self.x * self.x + self.y * self.y).sqrt() }
    fn normalized(self) -> Vec2 {
        let l = self.length();
        Vec2::new(self.x / l, self.y / l)
    }
    fn dot(self, o: Vec2) -> f32 { self.x * o.x + self.y * o.y }
    fn rotate(self, angle: f32) -> Vec2 {
        let (s, c) = angle.sin_cos();
        Vec2::new(self.x * c - self.y * s, self.x * s + self.y * c)
    }
}

fn main() {
    let v = Vec2::new(3.0, 4.0);
    println!("{} {:?}", v.length(), v.normalized());
    println!("{}", v.dot(Vec2::new(0.1, 0.2)));
    let r = Vec2::new(1.0, 0.0).rotate(std::f32::consts::FRAC_PI_2);
    println!("{:?}", r);
    println!("{:.3} {:.3}", r.x, r.y);
    let mut pos = Vec2::new(0.0, 10.0);
    let mut vel = Vec2::new(1.5, 0.0);
    let g = -9.81f32;
    let dt = 1.0f32 / 60.0;
    for _ in 0..60 {
        vel.y += g * dt;
        pos.x += vel.x * dt;
        pos.y += vel.y * dt;
    }
    println!("{:?} {:?}", pos, vel);
    println!("{}", 1.0f32 / 3.0 + 1.0 / 3.0);
    println!("{}", 100.0f32 * 1.1);
    println!("{}", 0.7f32 * 3.0);
    println!("{}", 16777217.0f32);
    println!("{}", 1.0e-40f32);
    println!("{:?}", 1.0e-40f32);
    println!("{:?}", 3.4e38f32 * 10.0);
    println!("{:?}", f32::MIN_POSITIVE);
    println!("{:?}", f32::EPSILON);
    println!("{}", (2.0f32).powf(0.5));
    println!("{}", (10.0f32).ln());
    println!("{}", (0.5f32).exp());
    println!("{}", 1e7f32);
    println!("{:?}", 1e7f32);
    println!("{:?}", 1.5e16f32);
    println!("{:?}", 123456.79f32);
    println!("{}", 0.1f32 + 0.7f32);
    let xs: Vec<f32> = vec![0.1, 0.2, 0.3, 0.4];
    let s: f32 = xs.iter().sum();
    println!("{}", s);
    let p: f32 = xs.iter().product();
    println!("{}", p);
    let avg = s / xs.len() as f32;
    println!("{}", avg);
    let var = xs.iter().map(|x| (x - avg) * (x - avg)).sum::<f32>() / xs.len() as f32;
    println!("{}", var);
    println!("{}", var.sqrt());
    let c = 37.5f32;
    let f = c * 9.0 / 5.0 + 32.0;
    println!("{}", f);
    println!("{}", (f - 32.0) * 5.0 / 9.0);
    println!("{}", 1.0f32.atan() * 4.0);
    println!("{}", (0.1f32 as f64) * 3.0);
    println!("{}", (0.1f64 as f32) * 3.0);
    println!("{}", 255.0f32 * 0.5f32);
    println!("{}", 2.0f32.mul_add(1.5, 0.25));
    println!("{}", (1.0f32 - 0.9).abs());
    println!("{}", -0.0f32);
    println!("{:?}", -0.0f32);
    println!("{}", 5.0f32.recip());
    println!("{}", 7.3f32.rem_euclid(2.0));
    println!("{}", f32::from_bits(0x40490fdb));
    println!("{:#x}", 1.5f32.to_bits());
    println!("{}", 1.5f32.to_degrees());
    println!("{}", 3.999999f32.floor());
    println!("{}", (0.1f32 + 0.2f32 == 0.3f32));
    println!("{}", (0.1f64 + 0.2f64 == 0.3f64));
    println!("{}", 12345.678f32.to_string());
    println!("{}", 12345.678f32.round());
    println!("{:8.2}|", 3.14159f32);
    println!("{:e}", 0.00123f32);
    println!("{:e}", 123456.0f64);
    println!("{:E}", 1.5e-10f64);
    println!("{:.2e}", 123456.0f64);
}
