fn main() {
    println!("cargo:rerun-if-changed=src/user.ld");
    println!("cargo:rustc-link-arg=-Tsrc/user.ld");
    println!("cargo::rustc-link-arg=-Map=user.map");
}
