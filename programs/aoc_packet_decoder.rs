// Day 16 2021: hex -> bit stream, recursive packet decoder.
struct Bits {
    bits: Vec<u8>,
    pos: usize,
}

impl Bits {
    fn from_hex(hex: &str) -> Self {
        let bits = hex
            .trim()
            .chars()
            .flat_map(|c| {
                let v = c.to_digit(16).unwrap();
                (0..4).rev().map(move |i| ((v >> i) & 1) as u8)
            })
            .collect();
        Bits { bits, pos: 0 }
    }

    fn take(&mut self, n: usize) -> u64 {
        let v = self.bits[self.pos..self.pos + n]
            .iter()
            .fold(0u64, |acc, &b| (acc << 1) | b as u64);
        self.pos += n;
        v
    }
}

#[derive(Debug)]
enum Payload {
    Literal(u64),
    Operator(u8, Vec<Packet>),
}

#[derive(Debug)]
struct Packet {
    version: u8,
    payload: Payload,
}

fn parse(b: &mut Bits) -> Packet {
    let version = b.take(3) as u8;
    let type_id = b.take(3) as u8;
    let payload = if type_id == 4 {
        let mut value = 0;
        loop {
            let more = b.take(1) == 1;
            value = (value << 4) | b.take(4);
            if !more {
                break;
            }
        }
        Payload::Literal(value)
    } else {
        let mut subs = Vec::new();
        if b.take(1) == 0 {
            let len = b.take(15) as usize;
            let end = b.pos + len;
            while b.pos < end {
                subs.push(parse(b));
            }
        } else {
            let count = b.take(11);
            for _ in 0..count {
                subs.push(parse(b));
            }
        }
        Payload::Operator(type_id, subs)
    };
    Packet { version, payload }
}

fn version_sum(p: &Packet) -> u64 {
    p.version as u64
        + match &p.payload {
            Payload::Literal(_) => 0,
            Payload::Operator(_, subs) => subs.iter().map(version_sum).sum(),
        }
}

fn eval(p: &Packet) -> u64 {
    match &p.payload {
        Payload::Literal(v) => *v,
        Payload::Operator(op, subs) => {
            let mut vals = subs.iter().map(eval);
            match op {
                0 => vals.sum(),
                1 => vals.product(),
                2 => vals.min().unwrap(),
                3 => vals.max().unwrap(),
                5 => (vals.next() > vals.next()) as u64,
                6 => (vals.next() < vals.next()) as u64,
                7 => (vals.next() == vals.next()) as u64,
                _ => unreachable!(),
            }
        }
    }
}

const INPUTS: &str = r"D2FE28
38006F45291200
8A004A801A8002F478
620080001611562C8802118E34
C0015000016115A2E0802F182340
A0016C880162017C3686B18A3D4780
C200B40A82
04005AC33890
880086C3E88112
CE00C43D881120
D8005AC2A8F0
F600BC2D8F
9C005AC2F8F0
9C0141080250320F1802104A08";

fn main() {
    for line in INPUTS.lines() {
        let mut bits = Bits::from_hex(line);
        let packet = parse(&mut bits);
        println!("{:<30} versions={:<3} value={}", line, version_sum(&packet), eval(&packet));
    }
    let mut b = Bits::from_hex("D2FE28");
    println!("{:?}", parse(&mut b));
}
