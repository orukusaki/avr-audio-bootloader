use crc::{Crc, CRC_16_XMODEM};

const PROGCOMMAND: u8 = 2;
const RUNCOMMAND: u8 = 3;

#[derive(Debug, PartialEq, Eq)]
pub struct Frame {
    command: u8,
    offset: u16,
    page: Vec<u8>,
}

impl From<Frame> for Vec<u8> {
    fn from(f: Frame) -> Vec<u8> {
        let mut v = vec![0, 0, 0, 1];
        v.push(f.command);

        v.append(&mut f.offset.to_le_bytes().to_vec());
        v.append(&mut f.page.to_vec());

        let crc = Crc::<u16>::new(&CRC_16_XMODEM);
        let checksum = crc.checksum(&v[4..]);
        println!("address: {:#06x} crc: {:#06x}", f.offset, checksum);
        v.append(&mut checksum.to_le_bytes().to_vec());
        v
    }
}

impl Frame {
    pub fn bytes_to_frames(bytes: &'_ [u8], page_size: usize) -> impl Iterator<Item = Frame> + '_ {
        bytes
            .chunks(page_size)
            .enumerate()
            .map(move |(frame_number, chunk)| {
                Frame::data((page_size * frame_number) as u16, chunk, page_size)
            })
    }

    pub fn data(offset: u16, bytes: &[u8], page_size: usize) -> Frame {
        let mut page = bytes.to_vec();
        while page.len() < page_size {
            page.push(0xff);
        }

        Frame {
            command: PROGCOMMAND,
            offset,
            page,
        }
    }

    pub fn run(page_size: usize) -> Frame {
        Frame {
            command: RUNCOMMAND,
            offset: 0,
            page: vec![0; page_size],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_frame_pads_to_length() {
        let frame = Frame::data(10, &[1; 2], 10);
        let expected = Frame {
            command: 2,
            offset: 10,
            page: vec![1, 1, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
        };
        assert_eq!(expected, frame);
    }

    #[test]
    fn test_run_frame_pads_to_length() {
        let frame = Frame::run(10);
        let expected = Frame {
            command: 3,
            offset: 0,
            page: vec![0; 10],
        };
        assert_eq!(expected, frame);
    }

    #[test]
    fn test_bytes_are_split_into_numbered_frames() {
        let bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let page_size = 6;
        let frames: Vec<Frame> = Frame::bytes_to_frames(&bytes, page_size).collect();

        let expected = vec![
            Frame {
                command: 2,
                offset: 0,
                page: vec![1, 2, 3, 4, 5, 6],
            },
            Frame {
                command: 2,
                offset: 6,
                page: vec![7, 8, 9, 10, 255, 255],
            },
        ];

        assert_eq!(expected, frames);
    }

    #[test]
    fn frame_is_rendered_as_bytes_with_crc() {
        let frame = Frame {
            command: 2,
            offset: 0,
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
