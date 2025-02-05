use std::{fs, time};
use encoding_rs::{EUC_JP, UTF_8};

const SOURCE: &'static str = "html/original_music_list_eucjp.html";
const DEST: &'static str = "html/original_music_list_utf8.html";

fn main() {
    let start = time::Instant::now();
    
    let raw = match fs::read(SOURCE) {
        Ok(v) => v,
        Err(e) => panic!("{e}") 
    };
    println!("buffer is {}bytes", raw.len());

    let (pointer, _encoding, has_error) = EUC_JP.decode(&raw);
    if has_error {
        panic!("failed to decode euc-jp encoded file");
    }

    let (pointer, _encoding, has_error) = UTF_8.encode(&pointer);
    if has_error {
        panic!("failed to encode to utf-8");
    }

    match fs::write(DEST, pointer) {
        Ok(()) => (),
        Err(e) => panic!("{e}")
    };

    println!("done in {}milsecs", start.elapsed().as_millis());
}
