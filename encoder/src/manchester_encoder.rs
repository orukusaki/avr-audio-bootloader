use super::frame::Frame;

#[derive(Default)]
pub struct ManchesterEncoder {
    state: u8,
}

impl ManchesterEncoder {
    pub fn new() -> ManchesterEncoder {
        ManchesterEncoder { state: 0 }
    }

    fn encode(&mut self, byte: u8) -> Vec<u8> {
        let mut v = Vec::new();
        let mut b = byte;

        for _ in 0..8 {
            self.state = !self.state;
            v.push(self.state);
            v.push(self.state);

            if b & 0x80 != 0 {
                self.state = !self.state;
            }

            v.push(self.state);
            v.push(self.state);

            b <<= 1;
        }

        v
    }

    pub fn encode_frame(&mut self, frame: Frame) -> Vec<u8> {
        let frame_bytes: Vec<u8> = frame.into();

        vec![127u8; 500]
            .into_iter()
            .chain(frame_bytes.into_iter().flat_map(|b| self.encode(b)))
            .collect::<Vec<u8>>()
            .into_iter()
            .chain(self.stop().into_iter())
            .collect()
    }

    pub fn stop(&mut self) -> Vec<u8> {
        self.state = !self.state;
        vec![self.state, self.state]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_0_byte() {
        let mut encoder = ManchesterEncoder::new();
        let expected = vec![
            0xff, 0xff, 0xff, 0xff, // 0
            0x00, 0x00, 0x00, 0x00, // 0
            0xff, 0xff, 0xff, 0xff, // 0
            0x00, 0x00, 0x00, 0x00, // 0
            0xff, 0xff, 0xff, 0xff, // 0
            0x00, 0x00, 0x00, 0x00, // 0
            0xff, 0xff, 0xff, 0xff, // 0
            0x00, 0x00, 0x00, 0x00, // 0
        ];

        assert_eq!(expected, encoder.encode(0x00));
    }

    #[test]
    fn test_encode_ff_byte() {
        let mut encoder = ManchesterEncoder::new();
        let expected = vec![
            0xff, 0xff, 0x00, 0x00, // 1
            0xff, 0xff, 0x00, 0x00, // 1
            0xff, 0xff, 0x00, 0x00, // 1
            0xff, 0xff, 0x00, 0x00, // 1
            0xff, 0xff, 0x00, 0x00, // 1
            0xff, 0xff, 0x00, 0x00, // 1
            0xff, 0xff, 0x00, 0x00, // 1
            0xff, 0xff, 0x00, 0x00, // 1
        ];

        assert_eq!(expected, encoder.encode(0xff));
    }

    #[test]
    fn test_encode_two_aa_bytes() {
        let mut encoder = ManchesterEncoder::new();
        let expected = vec![
            0xff, 0xff, 0x00, 0x00, // 1
            0xff, 0xff, 0xff, 0xff, // 0
            0x00, 0x00, 0xff, 0xff, // 1
            0x00, 0x00, 0x00, 0x00, // 0
            0xff, 0xff, 0x00, 0x00, // 1
            0xff, 0xff, 0xff, 0xff, // 0
            0x00, 0x00, 0xff, 0xff, // 1
            0x00, 0x00, 0x00, 0x00, // 0
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
            0xff, 0xff, 0x00, 0x00, // 1
            0xff, 0xff, 0xff, 0xff, // 0
            0x00, 0x00, 0xff, 0xff, // 1
            0x00, 0x00, 0x00, 0x00, // 0
            0xff, 0xff, 0x00, 0x00, // 1
            0xff, 0xff, 0xff, 0xff, // 0
            0x00, 0x00, 0xff, 0xff, // 1
            0x00, 0x00, 0xff, 0xff, // 1
        ];

        assert_eq!(expected1, encoder.encode(0xab));

        let expected2 = vec![
            0x00, 0x00, 0xff, 0xff, // 1
            0x00, 0x00, 0x00, 0x00, // 0
            0xff, 0xff, 0x00, 0x00, // 1
            0xff, 0xff, 0xff, 0xff, // 0
            0x00, 0x00, 0xff, 0xff, // 1
            0x00, 0x00, 0x00, 0x00, // 0
            0xff, 0xff, 0x00, 0x00, // 1
            0xff, 0xff, 0x00, 0x00, // 1
        ];

        assert_eq!(expected2, encoder.encode(0xab));
    }

    #[test]
    fn test_stop() {
        let mut encoder = ManchesterEncoder::new();
        assert_eq!(vec![0xff, 0xff], encoder.stop());
    }
}
