fn main() {
    println!("cargo:rerun-if-changed=src/kernel.ld");
    println!("cargo::rustc-link-arg=-Tsrc/kernel.ld");
    println!("cargo::rustc-link-arg=-Map=kernel.map");
    println!("cargo:rerun-if-changed=./shell.bin.o");
    println!("cargo::rustc-link-arg=./shell.bin.o");
}
