fn main() {
    use std::env;

    use print_bytes::print_lossy;

    print_lossy(&env::args_os().nth(1).expect("missing argument"));
}
