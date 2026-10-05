#![allow(non_snake_case)]

use rand::RngExt;

pub fn _GenerateCode() -> String {
    let mut Rng = rand::rng();
    format!("{:06}", Rng.random_range(0..1_000_000u32))
}
