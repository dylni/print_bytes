fn main() {
    use std::env;

    use print_bytes::print_bytes;

    print_bytes(&env::args_os().nth(1).expect("missing argument"));
}
