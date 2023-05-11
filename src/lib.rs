#[allow(unused_variables)]
#[allow(unused_imports)]
pub mod base;
pub mod r#macro;

pub use base::Either;

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
                subsamples: [subsample_count.unwrap()] {
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

            offset_count: u32,
            offsets: [offset_count] {
                offset: [u32, u64] [if version == 1], // u64 if version == 1, u32 if version == 0
            },
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
        Ftyp {
            major_brand: [u8; 4],
            minor_version: u32,
            compatible_brands: Vec<[u8; 4]>,
        },
        Moov : Container,
        Mvhd : Full {
            creation_time: [u32, u64] [if version == 1],
            modification_time: [u32, u64] [if version == 1],
            timescale: u32,
            duration: [u32, u64] [if version == 1],

            rate: i32,
            volume: i16,

            // 10 reserved bytes
            _reserved: [u8; 10],

            matrix: [i32; 9],
            pre_defined: [u32; 6],
            next_track_id: u32,
        },
        Trak : Container,
        Tkhd : Full {
            creation_time: [u32, u64] [if version == 1],
            modification_time: [u32, u64] [if version == 1],
            track_id: u32,

            // 4 reserved bytes
            _reserved: [u8; 4],

            duration: [u32, u64] [if version == 1],

            // 8 reserved bytes
            _reserved1: [u8; 8],

            layer: i16,
            alternate_group: i16,
            volume: i16,

            // 2 reserved bytes
            _reserved2: [u8; 2],

            matrix: [i32; 9],
            width: u32,
            height: u32,
        },
        Mdia : Container,
        Mdhd : Full {
            creation_time: [u32, u64] [if version == 1],
            modification_time: [u32, u64] [if version == 1],
            timescale: u32,
            duration: [u32, u64] [if version == 1],

            // Ignore first bit of language
            language: u16,

            // 2 reserved bytes
            _reserved1: [u8; 2],
        },
        Hdlr : Full {
            // 4 reserved bytes
            _reserved1: [u8; 4],

            handler_type: [u8; 4], // String

            // 12 reserved bytes
            _reserved2: [u8; 12],

            name: String,
        },
        Minf : Container,
        Smhd : Full {
            balance: i16,

            // 2 reserved bytes
            _reserved1: [u8; 2],
        },
        Dinf : Container,
        Dref : Skip,
        Stbl : Container,
        /* TODO: Implement this properly
        Stsd : Full {
            entry_count: u32,
            entries: [entry_count] {
                _size: u32,
                format: [u8; 4], // String

                // 6 reserved bytes
                _reserved: [u8; 6],

                data_reference_index: u16,
            },
        },
        */
        Stsd : Skip,
        Stts : Full {
            entry_count: u32,
            entries: [entry_count] {
                sample_count: u32,
                sample_delta: u32,
            },
        },
        Stsc : Full {
            entry_count: u32,
            entries: [entry_count] {
                first_chunk: u32,
                samples_per_chunk: u32,
                sample_description_index: u32,
            },
        },
        // TODO: Make this not an Optional vec & just have it be a vec of size 0 if cond not met
        Stsz : Full {
            sample_size: u32,
            sample_count: u32,
            entry_size: [sample_count] {
                size: u32,
            } [if sample_size == 0],
        },
        Stco : Full {
            entry_count: u32,
            chunk_offset: [entry_count] {
                offset: u32,
            },
        },
        Udta : Skip,
        Mvex : Container,
        Trex : Full {
            track_id: u32,
            default_sample_description_index: u32,
            default_sample_duration: u32,
            default_sample_size: u32,
            default_sample_flags: u32,
        },
        Pssh : Skip,
        Free : Skip,
        Edts : Skip,
        Sgpd : Skip,
        Sbgp : Skip,
    }
}

use base::*;
use boxes::*;

pub use boxes::Mp4Box;

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

pub fn write_mp4(boxes: &[Mp4Box]) -> Vec<u8> {
    let mut buf = vec![];

    for box_ in boxes {
        box_.write(&mut buf);
    }

    buf
}

// recursive search for box_type
pub fn find_box_mut<'a>(boxes: &'a mut [Mp4Box], box_type: &'a [u8; 4]) -> Option<&'a mut Mp4Box> {
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
                    next_search.push(box_.data.as_mut_slice());
                }
                Mp4Box::Traf(box_) => {
                    next_search.push(box_.data.as_mut_slice());
                }
                Mp4Box::Moov(box_) => {
                    next_search.push(box_.data.as_mut_slice());
                }
                Mp4Box::Trak(box_) => {
                    next_search.push(box_.data.as_mut_slice());
                }
                Mp4Box::Mdia(box_) => {
                    next_search.push(box_.data.as_mut_slice());
                }
                Mp4Box::Minf(box_) => {
                    next_search.push(box_.data.as_mut_slice());
                }
                Mp4Box::Dinf(box_) => {
                    next_search.push(box_.data.as_mut_slice());
                }
                Mp4Box::Stbl(box_) => {
                    next_search.push(box_.data.as_mut_slice());
                }
                Mp4Box::Mvex(box_) => {
                    next_search.push(box_.data.as_mut_slice());
                }
                _ => {}
            }
        }
    }

    None
}

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
                    next_search.push(box_.data.as_slice());
                }
                Mp4Box::Traf(box_) => {
                    next_search.push(box_.data.as_slice());
                }
                Mp4Box::Moov(box_) => {
                    next_search.push(box_.data.as_slice());
                }
                Mp4Box::Trak(box_) => {
                    next_search.push(box_.data.as_slice());
                }
                Mp4Box::Mdia(box_) => {
                    next_search.push(box_.data.as_slice());
                }
                Mp4Box::Minf(box_) => {
                    next_search.push(box_.data.as_slice());
                }
                Mp4Box::Dinf(box_) => {
                    next_search.push(box_.data.as_slice());
                }
                Mp4Box::Stbl(box_) => {
                    next_search.push(box_.data.as_slice());
                }
                Mp4Box::Mvex(box_) => {
                    next_search.push(box_.data.as_slice());
                }
                _ => {}
            }
        }
    }

    None
}

pub fn list_box_tree(boxes: &[Mp4Box], indent: usize) {
    for box_ in boxes {
        let name = get_box_type(box_);
        println!("{:indent$}{}", "", name, indent = indent * 2);

        match box_ {
            Mp4Box::Moof(box_) => {
                list_box_tree(&box_.data, indent + 1);
            }
            Mp4Box::Traf(box_) => {
                list_box_tree(&box_.data, indent + 1);
            }
            Mp4Box::Moov(box_) => {
                list_box_tree(&box_.data, indent + 1);
            }
            Mp4Box::Trak(box_) => {
                list_box_tree(&box_.data, indent + 1);
            }
            Mp4Box::Mdia(box_) => {
                list_box_tree(&box_.data, indent + 1);
            }
            Mp4Box::Minf(box_) => {
                list_box_tree(&box_.data, indent + 1);
            }
            Mp4Box::Dinf(box_) => {
                list_box_tree(&box_.data, indent + 1);
            }
            Mp4Box::Stbl(box_) => {
                list_box_tree(&box_.data, indent + 1);
            }
            Mp4Box::Mvex(box_) => {
                list_box_tree(&box_.data, indent + 1);
            }
            _ => {}
        }
    }
}
