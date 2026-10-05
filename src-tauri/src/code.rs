#![allow(non_snake_case)]

use rand::Rng;

pub fn _GenerateCode() -> String {
    let mut Rng = rand::thread_rng();
    format!("{:06}", Rng.gen_range(0..1_000_000u32))
}
