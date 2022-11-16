use super::frame::Frame;

#[derive(Default)]
pub struct ManchesterEncoder {
    state: bool,
}

const ONE_POSITIVE: [u8; 4] = [0x7f, 0xff, 0x7f, 0x00];
const ONE_NEGATIVE: [u8; 4] = [0x7f, 0x00, 0x7f, 0xff];
const ZERO_POSITIVE: [u8; 4] = [0x7f, 0xd9, 0xff, 0xd9];
const ZERO_NEGATIVE: [u8; 4] = [0x7f, 0x25, 0x00, 0x25];

impl ManchesterEncoder {
    pub fn new() -> ManchesterEncoder {
        ManchesterEncoder { state: false }
    }

    fn encode(&mut self, byte: u8) -> Vec<u8> {
        let mut v = Vec::new();
        let mut b = byte;

        for _ in 0..8 {
            let bit_value: bool = b & 0x80 != 0;

            let wave = match (self.state, bit_value) {
                (false, false) => ZERO_NEGATIVE,
                (false, true) => ONE_NEGATIVE,
                (true, false) => ZERO_POSITIVE,
                (true, true) => ONE_POSITIVE,
            };

            v.append(&mut wave.to_vec());

            if !bit_value {
                self.state = !self.state;
            }

            b <<= 1;
        }

        v
    }

    pub fn encode_frame(&mut self, frame: Frame) -> Vec<u8> {
        let frame_bytes: Vec<u8> = frame.into();

        vec![0x7fu8; 500]
            .into_iter()
            .chain(frame_bytes.into_iter().flat_map(|b| self.encode(b)))
            .collect::<Vec<u8>>()
            .into_iter()
            .chain(self.stop().into_iter())
            .collect()
    }

    pub fn stop(&mut self) -> Vec<u8> {
        self.state = !self.state;
        match self.state {
            false => vec![0x7f, 0xff],
            true => vec![0x7f, 0],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_0_byte() {
        let mut encoder = ManchesterEncoder::new();
        let expected = vec![
            0x7f, 0x25, 0x00, 0x25, // 0
            0x7f, 0xd9, 0xff, 0xd9, // 0
            0x7f, 0x25, 0x00, 0x25, // 0
            0x7f, 0xd9, 0xff, 0xd9, // 0
            0x7f, 0x25, 0x00, 0x25, // 0
            0x7f, 0xd9, 0xff, 0xd9, // 0
            0x7f, 0x25, 0x00, 0x25, // 0
            0x7f, 0xd9, 0xff, 0xd9, // 0
        ];

        assert_eq!(expected, encoder.encode(0x00));
    }

    #[test]
    fn test_encode_ff_byte() {
        let mut encoder = ManchesterEncoder::new();
        let expected = vec![
            0x7f, 0x00, 0x7f, 0xff, // 1
            0x7f, 0x00, 0x7f, 0xff, // 1
            0x7f, 0x00, 0x7f, 0xff, // 1
            0x7f, 0x00, 0x7f, 0xff, // 1
            0x7f, 0x00, 0x7f, 0xff, // 1
            0x7f, 0x00, 0x7f, 0xff, // 1
            0x7f, 0x00, 0x7f, 0xff, // 1
            0x7f, 0x00, 0x7f, 0xff, // 1
        ];

        assert_eq!(expected, encoder.encode(0xff));
    }

    #[test]
    fn test_encode_two_aa_bytes() {
        let mut encoder = ManchesterEncoder::new();
        let expected = vec![
            0x7f, 0x00, 0x7f, 0xff, // 1
            0x7f, 0x25, 0x00, 0x25, // 0
            0x7f, 0xff, 0x7f, 0x00, // 1
            0x7f, 0xd9, 0xff, 0xd9, // 0
            0x7f, 0x00, 0x7f, 0xff, // 1
            0x7f, 0x25, 0x00, 0x25, // 0
            0x7f, 0xff, 0x7f, 0x00, // 1
            0x7f, 0xd9, 0xff, 0xd9, // 0
        ];

        assert_eq!(expected, encoder.encode(0xaa));
        assert_eq!(expected, encoder.encode(0xaa));
    }

    #[test]
    fn test_encode_two_ab_bytes() {
        // The 2nd byte is inverted because of the state the encoder is left in
        // after the first byte
        let mut encoder = ManchesterEncoder::new();
        let expected1 = vec![
            0x7f, 0x00, 0x7f, 0xff, // 1
            0x7f, 0x25, 0x00, 0x25, // 0
            0x7f, 0xff, 0x7f, 0x00, // 1
            0x7f, 0xd9, 0xff, 0xd9, // 0
            0x7f, 0x00, 0x7f, 0xff, // 1
            0x7f, 0x25, 0x00, 0x25, // 0
            0x7f, 0xff, 0x7f, 0x00, // 1
            0x7f, 0xff, 0x7f, 0x00, // 1
        ];

        assert_eq!(expected1, encoder.encode(0xab));

        let expected2 = vec![
            0x7f, 0xff, 0x7f, 0x00, // 1
            0x7f, 0xd9, 0xff, 0xd9, // 0
            0x7f, 0x00, 0x7f, 0xff, // 1
            0x7f, 0x25, 0x00, 0x25, // 0
            0x7f, 0xff, 0x7f, 0x00, // 1
            0x7f, 0xd9, 0xff, 0xd9, // 0
            0x7f, 0x00, 0x7f, 0xff, // 1
            0x7f, 0x00, 0x7f, 0xff, // 1
        ];

        assert_eq!(expected2, encoder.encode(0xab));
    }

    #[test]
    fn test_stop() {
        let mut encoder = ManchesterEncoder::new();
        assert_eq!(vec![0x7f, 0x00], encoder.stop());
    }
}
