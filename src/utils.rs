#[macro_export]
macro_rules! fixed_vec {
    ($n:expr) => {
        Vec::with_capacity($n)
    };
}
