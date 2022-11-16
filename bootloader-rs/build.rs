fn main() {
    println!("cargo:rustc-link-arg=-lc");
    println!("cargo:rustc-link-arg-bin=main=-Wl,--section-start=.text=0x7C00,-lc");
    println!("cargo:rustc-env=AVR_CPU_FREQUENCY_HZ=16000000");
}
