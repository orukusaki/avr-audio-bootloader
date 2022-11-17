fn main() {
    println!("cargo:rustc-link-arg=-lc");
    println!("cargo:rustc-link-arg-bin=main=-Wl,--section-start=.text=0x7C00,-lc");
    println!("cargo:rustc-env=AVR_CPU_FREQUENCY_HZ=16000000");
    println!(
        "cargo:rustc-env=SPM_PAGESIZE={}",
        get_spm_page_size().expect("Could not find spm page size")
    );
}

fn get_spm_page_size() -> Option<u32> {
    let current_mcu = avr_mcu::current::mcu().expect("no target cpu specified");

    current_mcu
        .device
        .address_spaces
        .iter()
        .find(|space| space.name == "prog")
        .and_then(|space| space.segments.first())
        .and_then(|seg| seg.page_size)
}
