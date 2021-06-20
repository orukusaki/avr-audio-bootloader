use crc::{Crc, CRC_16_XMODEM};

const PROGCOMMAND: u8 = 2;
const RUNCOMMAND: u8 = 3;

#[derive(Debug, PartialEq)]
pub struct Frame {
    command: u8,
    page_index: u16,
    page: Vec<u8>,
}

impl From<Frame> for Vec<u8> {
    fn from(f: Frame) -> Vec<u8> {
        let mut v = vec![0, 0, 0, 1];
        v.push(f.command);
        v.push((f.page_index & 0xff) as u8);
        v.push((f.page_index >> 8) as u8);
        v.append(&mut f.page.to_vec());

        let crc = Crc::<u16>::new(&CRC_16_XMODEM);
        let checksum = crc.checksum(&v[4..]);

        v.push((checksum & 0xff) as u8);
        v.push((checksum >> 8) as u8);
        v
    }
}

impl Frame {
    pub fn bytes_to_frames(bytes: &'_ [u8], frame_size: usize) -> impl Iterator<Item = Frame> + '_ {
        bytes
            .chunks(frame_size)
            .enumerate()
            .map(move |(frame_number, chunk)| Frame::data(frame_number as u16, chunk, frame_size))
    }

    pub fn data(page_index: u16, bytes: &[u8], frame_size: usize) -> Frame {
        let mut page = bytes.to_vec();
        while page.len() < frame_size {
            page.push(0xff);
        }

        Frame {
            command: PROGCOMMAND,
            page_index,
            page,
        }
    }

    pub fn run(frame_size: usize) -> Frame {
        Frame {
            command: RUNCOMMAND,
            page_index: 0,
            page: vec![0; frame_size],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_frame_pads_to_length() {
        let frame = Frame::data(1, &[1; 2], 10);
        let expected = Frame {
            command: 2,
            page_index: 1,
            page: vec![1, 1, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
        };
        assert_eq!(expected, frame);
    }

    #[test]
    fn test_run_frame_pads_to_length() {
        let frame = Frame::run(10);
        let expected = Frame {
            command: 3,
            page_index: 0,
            page: vec![0; 10],
        };
        assert_eq!(expected, frame);
    }

    #[test]
    fn test_bytes_are_split_into_numbered_frames() {
        let bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let frame_size = 6;
        let frames: Vec<Frame> = Frame::bytes_to_frames(&bytes, frame_size).collect();

        let expected = vec![
            Frame {
                command: 2,
                page_index: 0,
                page: vec![1, 2, 3, 4, 5, 6],
            },
            Frame {
                command: 2,
                page_index: 1,
                page: vec![7, 8, 9, 10, 255, 255],
            },
        ];

        assert_eq!(expected, frames);
    }

    #[test]
    fn frame_is_rendered_as_bytes_with_crc() {
        let frame = Frame {
            command: 2,
            page_index: 0,
            page: vec![0; 128],
        };

        let head = vec![0, 0, 0, 1, 2, 0, 0];
        let mut body = vec![0; 128];
        let mut tail = vec![0xAF, 0xF2];

        let mut expected: Vec<u8> = head;
        expected.append(&mut body);
        expected.append(&mut tail);

        let bytes: Vec<u8> = frame.into();

        assert_eq!(expected, bytes);
    }
}
