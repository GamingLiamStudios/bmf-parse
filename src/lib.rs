mod base;
pub mod r#macro;

pub mod boxes {
    use crate::r#macro::mp4box_gen;

    mp4box_gen! { flags version;
        Moof : Container,
        Mfhd : Full {
            seq_num: u32,
        },
        Traf : Container,
        Tfhd : Full {
            track_id: u32,
            base_data_offset: u64 [if flags & 0x000001 != 0],
            sample_description_index: u32 [if flags & 0x000002 != 0],
            default_sample_duration: u32 [if flags & 0x000008 != 0],
            default_sample_size: u32 [if flags & 0x000010 != 0],
            default_sample_flags: u32 [if flags & 0x000020 != 0]
        },
        Tfdt : Full {
            base_media_decode_time: u64,
        },
        /*
        Senc : Full {
            samples: [u32] {
                iv: [u8; 8],
                subsamples: [u16] {
                    clear_bytes: u16,
                    cipher_bytes: u32,
                } [if flags & 0x000002]
            }
        },
        */
    }
}

use base::*;
use boxes::*;

pub fn parse_mp4(input: &[u8]) -> Vec<Mp4Box> {
    let mut state = ParserState { offset: 0 };
    let mut boxes = vec![];

    while !is_empty(input, &state) {
        if let Some(box_) = parse_box(input, &mut state) {
            boxes.push(box_);
        }
    }

    boxes
}

// recursive search for box_type
pub fn find_box<'a>(boxes: &'a [Mp4Box], box_type: &'a [u8; 4]) -> Option<&'a Mp4Box> {
    let box_type = u32::from_ne_bytes(*box_type);

    let mut next_search = vec![boxes];

    while let Some(boxes) = next_search.pop() {
        for box_ in boxes {
            if is_box_type(box_, box_type) {
                return Some(box_);
            }

            // TODO: macro-ify this part
            match box_ {
                Mp4Box::Moof(box_) => {
                    next_search.push(&box_.data);
                }
                Mp4Box::Traf(box_) => {
                    next_search.push(&box_.data);
                }
                _ => {}
            }
        }
    }

    None
}
