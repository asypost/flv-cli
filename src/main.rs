#[allow(dead_code)]
mod flv;

use flv::{Header, Segment};
use std::{fs::File, io::Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        match File::open(args.get(1).unwrap()) {
            Ok(mut file) => {
                let header = Header::from_reader(&mut file).unwrap();
                println!("signature: {}", header.signature());
                println!("version: {}", header.version());
                println!("video: {}", header.has_video());
                println!("audio: {}", header.has_audio());
                println!("header size: {}", header.size());
                let mut of = File::create("D:/out.flv").unwrap();

                of.write_all(&header.into_bytes()).unwrap();
                let mut pre_tag_size = 0_u32;
                loop {
                    match Segment::from_reader(&mut file) {
                        Ok(mut item) => {
                            if !item.has_tag() || item.has_video_tag() || item.has_script_tag() {
                                item.set_pre_tag_size(pre_tag_size);
                                if item.has_tag() {
                                    pre_tag_size = item.tag().as_ref().unwrap().tag_size();
                                } else {
                                    pre_tag_size = 0_u32;
                                }
                                of.write_all(&item.into_bytes()).unwrap();
                            }
                            if item.has_tag() {
                                if item.has_audio_tag() {
                                    print!("tag type: audio ");
                                } else if item.has_video_tag() {
                                    print!("tag type: video ");
                                } else {
                                    print!("tag type: script ");
                                }
                                println!(
                                    "timestamp: {}",
                                    item.tag().as_ref().unwrap().timestamp()
                                )
                            }
                        }
                        Err(e) => {
                            eprintln!("{:?}", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    } else {
        println!("flv [文件路径]");
    }
}
