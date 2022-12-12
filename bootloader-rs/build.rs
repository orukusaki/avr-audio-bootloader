fn main() {
    println!("cargo:rustc-link-arg=-lc");
    println!("cargo:rustc-link-arg-bin=main=-Wl,--section-start=.text=0x7C00,-lc"); //TODO: some way of automatically setting the --section-start based on the bin size? :-/
}
