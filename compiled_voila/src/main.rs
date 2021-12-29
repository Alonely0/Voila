fn main() {
    // the use of the `env!()` macro instead of
    // its `option_env!()` counterpart is on purpose,
    // so if the data needed to run is not provided, a compile-time
    // error will be thrown instead of a runtime one.
    if let Err(ref e) = voila::exec(
        str_to_vec_u8(env!("v_code")).into(), // into() automatically deserializes the data
        std::path::PathBuf::from(env!("v_path")),
        env!("v_recursive").parse().unwrap(),
    ) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn str_to_vec_u8(mut str: &str) -> Vec<u8> {
    str = str.strip_prefix('[').unwrap().strip_suffix(']').unwrap();
    str.split(", ").map(|n| n.parse::<u8>().unwrap()).collect()
}
