// Struct-heavy simulation: Vec<Particle> updated in place each step.
#[derive(Debug, Clone, Copy)]
struct Particle {
    x: i64,
    y: i64,
    vx: i64,
    vy: i64,
    bounces: u32,
}

impl Particle {
    fn step(&mut self, width: i64, height: i64) {
        self.x += self.vx;
        self.y += self.vy;
        if self.x < 0 || self.x >= width {
            self.vx = -self.vx;
            self.x += 2 * self.vx;
            self.bounces += 1;
        }
        if self.y < 0 || self.y >= height {
            self.vy = -self.vy;
            self.y += 2 * self.vy;
            self.bounces += 1;
        }
    }
}

fn main() {
    let (width, height) = (1000, 700);
    let mut particles: Vec<Particle> = (0..200)
        .map(|i| Particle { x: (i * 37) % width, y: (i * 91) % height, vx: i % 7 - 3, vy: i % 5 - 2, bounces: 0 })
        .collect();
    for _ in 0..150 {
        for p in particles.iter_mut() {
            p.step(width, height);
        }
        for i in 0..particles.len() {
            if particles[i].vx == 0 {
                particles[i].vx = 1;
            }
        }
    }
    let total: u32 = particles.iter().map(|p| p.bounces).sum();
    let cx: i64 = particles.iter().map(|p| p.x).sum::<i64>() / particles.len() as i64;
    let cy: i64 = particles.iter().map(|p| p.y).sum::<i64>() / particles.len() as i64;
    println!("bounces: {}", total);
    println!("centroid: ({}, {})", cx, cy);
    println!("{:?}", particles[17]);
    println!("{:?}", particles[199]);
}
