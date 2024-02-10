#[cfg(feature = "os_str_bytes")]
use std::env;

#[cfg(feature = "os_str_bytes")]
use print_bytes::print_lossy;

fn main() {
    #[cfg(feature = "os_str_bytes")]
    print_lossy(&env::args_os().nth(1).expect("missing argument"));
}
