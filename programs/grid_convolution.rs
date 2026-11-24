// Image convolution on Vec<Vec<f64>> with fixed-size kernels.
type Image = Vec<Vec<f64>>;

fn convolve(img: &Image, kernel: &[[f64; 3]; 3]) -> Image {
    let h = img.len();
    let w = img[0].len();
    let mut out = vec![vec![0.0; w]; h];
    for y in 0..h {
        for x in 0..w {
            let mut acc = 0.0;
            for (ky, krow) in kernel.iter().enumerate() {
                for (kx, k) in krow.iter().enumerate() {
                    let sy = (y as i64 + ky as i64 - 1).clamp(0, h as i64 - 1) as usize;
                    let sx = (x as i64 + kx as i64 - 1).clamp(0, w as i64 - 1) as usize;
                    acc += img[sy][sx] * k;
                }
            }
            out[y][x] = acc;
        }
    }
    out
}

fn print(name: &str, img: &Image) {
    println!("{}", name);
    for row in img {
        let cells: Vec<String> = row.iter().map(|v| format!("{:6.2}", v)).collect();
        println!("{}", cells.join(" "));
    }
}

fn ascii(img: &Image) {
    let ramp = [' ', '.', ':', '*', '#'];
    let max = img.iter().flatten().cloned().fold(f64::MIN, f64::max);
    let min = img.iter().flatten().cloned().fold(f64::MAX, f64::min);
    for row in img {
        let line: String = row
            .iter()
            .map(|&v| {
                let t = if max > min { (v - min) / (max - min) } else { 0.0 };
                ramp[((t * 4.0).round() as usize).min(4)]
            })
            .collect();
        println!("|{}|", line);
    }
}

fn main() {
    let mut img: Image = vec![vec![0.0; 7]; 6];
    for y in 1..5 {
        for x in 2..6 {
            img[y][x] = 1.0;
        }
    }
    img[2][3] = 2.5;
    print("input", &img);

    let blur = [[1.0 / 9.0; 3]; 3];
    let sharpen = [[0.0, -1.0, 0.0], [-1.0, 5.0, -1.0], [0.0, -1.0, 0.0]];
    let edge = [[-1.0, -1.0, -1.0], [-1.0, 8.0, -1.0], [-1.0, -1.0, -1.0]];

    let b = convolve(&img, &blur);
    print("blur", &b);
    ascii(&b);
    let s = convolve(&img, &sharpen);
    print("sharpen", &s);
    let e = convolve(&img, &edge);
    ascii(&e);

    let total: f64 = img.iter().map(|r| r.iter().sum::<f64>()).sum();
    let btotal: f64 = b.iter().flat_map(|r| r.iter()).sum();
    println!("total {:.3} blurred {:.3}", total, btotal);
    let mean = btotal / (b.len() * b[0].len()) as f64;
    println!("mean {:.4}", mean);
    let (my, mx) = (0..img.len())
        .flat_map(|y| (0..img[0].len()).map(move |x| (y, x)))
        .max_by(|a, c| e[a.0][a.1].partial_cmp(&e[c.0][c.1]).unwrap())
        .unwrap();
    println!("strongest edge at ({}, {}) = {:.3}", my, mx, e[my][mx]);
    println!("{:?}", b[0].iter().map(|v| (v * 100.0).round() / 100.0).collect::<Vec<f64>>());
}
