use flate2::write::GzEncoder;
use flate2::Compression;
use std::env::current_dir;
use std::fs::File;

const COMPILE_ASSETS: [&str; 4] = ["voila", "compiled_voila", "Cargo.toml", "Cargo.lock"];

fn main() {
    let src = &current_dir().expect("Can't open current directory");
    let src = src.parent().unwrap();
    let dest = File::create(format!("{}/code.tar.gz", src.to_str().unwrap()))
        .expect("Can't write to current directory");
    println!("{}/code.tar.gz", src.to_str().unwrap());
    let encoder = GzEncoder::new(dest, Compression::default());
    let mut tar = tar::Builder::new(encoder);
    for a in COMPILE_ASSETS {
        let p = src.join(a);
        if p.is_dir() {
            tar.append_dir_all(a, p)
                .expect("Can't read current directory");
        } else {
            tar.append_path_with_name(p, a)
                .expect("Can't read current directory");
        }
    }
    tar.finish().unwrap();
}
