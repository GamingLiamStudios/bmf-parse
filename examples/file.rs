use std::io::{Read, Write};

use bmf_parse::{boxes::*, *};

fn main() {
    let file = std::fs::File::open("dec.mp4").unwrap();

    // Read file into a slice
    let buf = file.bytes().map(|b| b.unwrap()).collect::<Vec<_>>();

    let mp4 = parse_mp4(&buf);

    list_box_tree(mp4.as_slice(), 0);

    // write the mp4 back to a file
    let mut file = std::fs::File::create("enc.mp4").unwrap();

    let buf = write_mp4(&mp4);

    file.write_all(&buf).unwrap();
}
