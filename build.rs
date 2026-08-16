use std::env;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    if let Ok(dir) = env::var("RYZENADJ_LIB_DIR") {
        println!("cargo:rustc-link-search=native={dir}");
    }
    println!("cargo:rustc-link-search=native={manifest_dir}/lib");
    println!("cargo:rustc-link-search=native=/usr/local/lib");

    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
    println!("cargo:rerun-if-env-changed=RYZENADJ_LIB_DIR");
}
