// Arbitrary-precision unsigned arithmetic on u32 limbs, the way people write it without crates.
#[derive(Clone, Debug, PartialEq)]
struct BigUint {
    limbs: Vec<u32>,
}

impl BigUint {
    fn from_u64(mut n: u64) -> Self {
        let mut limbs = Vec::new();
        while n > 0 {
            limbs.push(n as u32);
            n >>= 32;
        }
        BigUint { limbs }
    }

    fn is_zero(&self) -> bool { self.limbs.is_empty() }

    fn add(&self, other: &BigUint) -> BigUint {
        let mut out = Vec::new();
        let mut carry = 0u64;
        for i in 0..self.limbs.len().max(other.limbs.len()) {
            let a = *self.limbs.get(i).unwrap_or(&0) as u64;
            let b = *other.limbs.get(i).unwrap_or(&0) as u64;
            let s = a + b + carry;
            out.push(s as u32);
            carry = s >> 32;
        }
        if carry > 0 { out.push(carry as u32); }
        BigUint { limbs: out }
    }

    fn mul_small(&self, k: u32) -> BigUint {
        let mut out = Vec::with_capacity(self.limbs.len() + 1);
        let mut carry = 0u64;
        for &limb in &self.limbs {
            let p = limb as u64 * k as u64 + carry;
            out.push((p & 0xFFFF_FFFF) as u32);
            carry = p >> 32;
        }
        if carry != 0 { out.push(carry as u32); }
        while out.last() == Some(&0) { out.pop(); }
        BigUint { limbs: out }
    }

    fn mul(&self, other: &BigUint) -> BigUint {
        let mut out = vec![0u32; self.limbs.len() + other.limbs.len()];
        for (i, &a) in self.limbs.iter().enumerate() {
            let mut carry = 0u64;
            for (j, &b) in other.limbs.iter().enumerate() {
                let t = out[i + j] as u64 + a as u64 * b as u64 + carry;
                out[i + j] = t as u32;
                carry = t >> 32;
            }
            let mut k = i + other.limbs.len();
            while carry > 0 {
                let t = out[k] as u64 + carry;
                out[k] = t as u32;
                carry = t >> 32;
                k += 1;
            }
        }
        while out.last() == Some(&0) { out.pop(); }
        BigUint { limbs: out }
    }

    fn divmod_small(&self, d: u32) -> (BigUint, u32) {
        let mut out = vec![0u32; self.limbs.len()];
        let mut rem = 0u64;
        for i in (0..self.limbs.len()).rev() {
            let cur = (rem << 32) | self.limbs[i] as u64;
            out[i] = (cur / d as u64) as u32;
            rem = cur % d as u64;
        }
        while out.last() == Some(&0) { out.pop(); }
        (BigUint { limbs: out }, rem as u32)
    }

    fn to_decimal(&self) -> String {
        if self.is_zero() { return "0".to_string(); }
        let mut digits = Vec::new();
        let mut cur = self.clone();
        while !cur.is_zero() {
            let (q, r) = cur.divmod_small(1_000_000_000);
            digits.push(r);
            cur = q;
        }
        let mut s = digits.pop().unwrap().to_string();
        while let Some(d) = digits.pop() {
            s.push_str(&format!("{:09}", d));
        }
        s
    }
}

fn factorial(n: u32) -> BigUint {
    let mut acc = BigUint::from_u64(1);
    for i in 2..=n { acc = acc.mul_small(i); }
    acc
}

fn main() {
    println!("{}", factorial(30).to_decimal());
    println!("{}", factorial(52).to_decimal());
    let a = BigUint::from_u64(u64::MAX);
    let b = a.mul(&a);
    println!("{}", b.to_decimal());
    println!("{:?}", b.limbs);
    println!("{}", (u64::MAX as u128 * u64::MAX as u128));
    let mut fib = (BigUint::from_u64(0), BigUint::from_u64(1));
    for _ in 0..200 {
        let next = fib.0.add(&fib.1);
        fib = (fib.1, next);
    }
    println!("{}", fib.0.to_decimal());
    let two = BigUint::from_u64(2);
    let mut p = BigUint::from_u64(1);
    for _ in 0..128 { p = p.mul(&two); }
    println!("{}", p.to_decimal());
    println!("{}", u128::MAX);
    // checked u128 factorial overflow boundary
    let mut f: u128 = 1;
    let mut n = 1u128;
    loop {
        match f.checked_mul(n) {
            Some(v) => { f = v; n += 1; }
            None => break,
        }
    }
    println!("{}! = {} fits, {}! overflows", n - 1, f, n);
}
