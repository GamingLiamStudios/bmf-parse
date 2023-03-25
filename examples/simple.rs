use bmf_parse::{boxes::*, *};

fn main() {
    let mp4 = parse_mp4(&[
        0x00, 0x00, 0x00, 0x18, b'm', b'o', b'o', b'f', 0x00, 0x00, 0x00, 0x10, b'm', b'f', b'h',
        b'd', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
    ]);
    println!("{:#?}", mp4);

    let moof = find_box(mp4.as_slice(), b"moof").unwrap();
    let moof = match moof {
        Mp4Box::Moof(moof) => &**moof,
        _ => panic!("Not a BoxMoof!"),
    };

    let mfhd = find_box(moof.data.as_slice(), b"mfhd").unwrap();
    let mfhd = match mfhd {
        Mp4Box::Mfhd(mfhd) => &**mfhd,
        _ => panic!("Not a BoxMfhd!"),
    };
}
