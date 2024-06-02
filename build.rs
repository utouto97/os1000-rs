fn main() {
    println!("cargo:rerun-if-changed=src/kernel.ld");
    println!("cargo::rustc-link-arg=-Tsrc/kernel.ld");
    println!("cargo::rustc-link-arg=-Map=kernel.map");
}
