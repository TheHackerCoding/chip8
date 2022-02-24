use rand::Rng;

#[macro_export]
macro_rules! fixed_vec {
    ($n:expr) => {
        Vec::with_capacity($n)
    };
}

pub fn random(range: u16) -> u16 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..range)
}
