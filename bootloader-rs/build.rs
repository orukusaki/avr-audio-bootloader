fn main() {
    println!("cargo:rustc-link-arg=-lc");
    println!("cargo:rustc-link-arg-bin=main=-Wl,--section-start=.text=0x7C00,-lc"); //TODO: some way of automatically setting the --section-start based on the bin size? :-/

    let (flash_size, page_size) = get_spm_page_size().expect("Could not find spm page size");
    let spm_variant = if flash_size > 0xffff {
        "extended"
    } else {
        "normal"
    };

    println!("cargo:rustc-env=SPM_PAGESIZE={}", page_size);
    println!("cargo:rustc-cfg=spm_variant=\"{}\"", spm_variant);
}

fn get_spm_page_size() -> Option<(u32, u32)> {
    let current_mcu = avr_mcu::current::mcu().expect("no target cpu specified");

    current_mcu
        .device
        .address_spaces
        .iter()
        .find(|space| space.name == "prog")
        .and_then(|space| space.segments.iter().find(|seg| seg.name == "FLASH"))
        .and_then(|seg| seg.page_size.map(|page_size| (seg.size, page_size)))
}
