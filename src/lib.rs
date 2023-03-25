#[allow(unused_variables)]
#[allow(unused_imports)]
mod base;
pub mod r#macro;

pub mod boxes {
    use crate::r#macro::mp4box_gen;

    mp4box_gen! { version flags;
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
        Senc : Full {
            sample_count: u32,
            samples: [sample_count] {
                iv: [u8; 8],
                subsample_count: u16 [if flags & 0x000002 != 0],
                subsamples: [subsample_count] {
                    clear_bytes: u16,
                    cipher_bytes: u32,
                } [if flags & 0x000002 != 0]
            }
        },
        Saiz : Full {
            aux_info_type: u32 [if flags & 0x000001 != 0],
            aux_info_type_parameter: u32 [if flags & 0x000001 != 0],

            default_sample_info_size: u8,
            sample_count: u32,
            sample_info_size: u8 [if default_sample_info_size == 0]
        },
        Saio : Full {
            aux_info_type: u32 [if flags & 0x000001 != 0],
            aux_info_type_parameter: u32 [if flags & 0x000001 != 0],

            entry_count: u32,
            offset: [u64, u32] [if version == 1], // u64 if version == 1, u32 if version == 0
        },
        Trun : Full {
            sample_count: u32,

            data_offset: i32 [if flags & 0x000001 != 0],
            first_sample_flags: u32 [if flags & 0x000004 != 0],

            samples: [sample_count] {
                sample_duration: u32 [if flags & 0x000100 != 0],
                sample_size: u32 [if flags & 0x000200 != 0],
                sample_flags: u32 [if flags & 0x000400 != 0],
                // i32 if version == 1, u32 if version == 0, only present if flags & 0x000800
                // [A, B, C..] [C.. condition] [BC condition] [ABC/present condition]
                sample_composition_time_offset: [u32, i32] [if version == 1] [if flags & 0x000800 != 0],
            }
        },
        Mdat : Container = u8,
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
