use core::arch::asm;

pub fn crc_xmodem_update(mut crc: u16, data: u8) -> u16 {
    crc ^= (data as u16) << 8;
    for _ in 0..8 {
        if crc & 0x8000 != 0 {
            crc = (crc << 1) ^ 0x1021;
        } else {
            crc <<= 1;
        }
    }

    crc
}

pub fn crc_xmodem_update_asm(crc: u16, data: u8) -> u16 {
    let mut ret: u16 = crc;

    unsafe {
        asm! (
            "eor    {crc:h},{data}"         , /* crc.hi ^ data */
            "mov    r0,{crc:h}",
            "swap   r0"    , /* swap(crc.hi ^ data) */

            /* Calculate the ret.lo of the CRC. */
            "mov    {tmp1},r0"  ,
            "andi   {tmp1},0x0f"         ,
            "eor    {tmp1},{crc:h}"          ,
            "mov    {tmp2},{crc:h}"          ,
            "eor    {tmp2},r0"  ,
            "lsl    {tmp2}"              ,
            "andi   {tmp2},0xe0"         ,
            "eor    {tmp1},{tmp2}"           , /* __tmp1 is now ret.lo. */

            /* Calculate the ret.hi of the CRC. */
            "mov    {tmp2},r0"  ,
            "eor    {tmp2},{crc:h}"          ,
            "andi   {tmp2},0xf0"         ,
            "lsr    {tmp2}"              ,
            "mov    r0,{crc:h}" ,
            "lsl    r0"     ,
            "rol    {tmp2}"              ,
            "lsr    {crc:h}"             ,
            "lsr    {crc:h}"             ,
            "lsr    {crc:h}"             ,
            "andi   {crc:h},0x1f"        ,
            "eor    {crc:h},{tmp2}"          ,
            "eor    {crc:h},{crc:l}"         , /* ret.hi is now ready. */
            "mov    {crc:l},{tmp1}"          , /* ret.lo is now ready. */
            data = in(reg) data,
            crc = inout(reg_iw) ret,
            tmp1 = out(reg_upper) _,
            tmp2 = out(reg_upper) _,
        )
    }
    ret
}
