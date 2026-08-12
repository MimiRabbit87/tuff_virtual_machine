use std::time::{SystemTime, UNIX_EPOCH};

const SH0: u8 = 1;
const SH1: u8 = 10;
const SH8: u8 = 8;
const MASK: u32 = 0b0111_1111_1111_1111_1111_1111_1111_1111;

pub struct TinyMT8Bit {
    s0: u32,
    s1: u32,
    s2: u32,
    s3: u32,
}

impl TinyMT8Bit {
    pub fn new() -> Self {
        let seed: u64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        let mut s0: u32 = seed as u32;
        let mut s1: u32 = seed.wrapping_add(0x9e3779b9) as u32;
        let mut s2: u32 = seed.wrapping_add(0x3c6ef372) as u32;
        let mut s3: u32 = seed.wrapping_add(0x85ebca6b) as u32;

        s0 = s0.wrapping_mul(0x9e3779b9).rotate_left(13);
        s1 = s1.wrapping_mul(0x85ebca6b).rotate_left(17);
        s2 = s2.wrapping_mul(0x3c6ef372).rotate_left(21);
        s3 = s3.wrapping_mul(0x9e3779b9).rotate_left(25);

        Self { s0, s1, s2, s3 }
    }
}

impl Iterator for TinyMT8Bit {
    type Item = u8;

    fn next(self: &mut Self) -> Option<Self::Item> {
        let t0: u32 = self.s0;
        let t1: u32 = self.s2;
        let mut x: u32 = (self.s0 & MASK) ^ self.s1 ^ self.s2;
        x = x ^ (x << SH0);
        let y: u32 = self.s3 ^ (self.s3 >> SH0) ^ x;
        self.s0 = self.s1;
        self.s1 = self.s2;
        self.s2 = x ^ (y << SH1);
        self.s3 = y;
        if y & 0b0000_0000_0000_0000_0000_0000_0000_0001 == 1 {
            self.s1 ^= 0b1010_0101_1111_0000_0011_1100_1001_0110;
            self.s2 ^= 0b1001_0110_0011_1100_1111_0000_1010_0101;
        };
        let t: u32 = t0 + (t1 >> SH8);
        Some(((t ^ (t >> 1) ^ 0b1110_0001_1100_0011_1000_0001_0000_0000) >> 0xF) as u8)
    }
}
