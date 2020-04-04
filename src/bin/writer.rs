fn main() {
    #[cfg(feature = "os_str")]
    {
        use std::env;

        use print_bytes::print_bytes;

        print_bytes(&env::args_os().nth(1).expect("missing argument"));
    }
    #[cfg(not(feature = "os_str"))]
    panic!("missing feature");
}
