use clap::{App, Arg, SubCommand};
use flv_parser::amf::amf0::Value;
use flv_parser::flv::{Header, Segment, TagData};
use std::{fs::File, io};

fn main() {
    let matches = App::new("flv cli")
        .version("0.1")
        .author("Shell asypost@gmail.com")
        .arg(Arg::with_name("FILE").help("flv file").required(true))
        .subcommand(
            SubCommand::with_name("info")
                .version("0.1")
                .about("Show flv file metadata"),
        )
        .subcommand(
            SubCommand::with_name("extract")
                .version("0.1")
                .about("Extract video or audio from flv file")
                .arg(
                    Arg::with_name("type")
                        .short("-t")
                        .long("--type")
                        .takes_value(true)
                        .required(true)
                        .help("audio,video or all"),
                )
                .arg(
                    Arg::with_name("output")
                        .short("-o")
                        .long("--out")
                        .takes_value(true)
                        .required(true)
                        .help("output path,- for stdout"),
                ),
        )
        .get_matches();

    if let Some(file) = matches.value_of("FILE") {
        if let Some(_) = matches.subcommand_matches("info") {
            show_flv_info(file).expect("Read flv file error");
        } else if let Some(args) = matches.subcommand_matches("extract") {
            let tp = args.value_of("type").unwrap();
            let out = args.value_of("output").unwrap();
            if tp != "audio" && tp != "video" && tp != "all" {
                println!("{}", args.usage());
            } else {
                extract(file, tp, out).expect("Extract failed");
            }
        }
    } else {
        println!("{}", matches.usage());
    }
}

fn extract(src: &str, tp: &str, path: &str) -> io::Result<()> {
    let mut fp = File::open(src)?;
    let header = Header::from_reader(&mut fp)?;
    let stdout = io::stdout();
    let mut ofp: Box<dyn io::Write> = if path == "-" {
        Box::new(stdout.lock())
    } else {
        let out = File::create(path)?;
        Box::new(out)
    };
    ofp.write_all(&header.into_bytes())?;
    let mut pre_tag_size = 0_u32;
    loop {
        if let Ok(mut seg) = Segment::from_reader(&mut fp) {
            if !seg.has_tag() {
                ofp.write_all(&seg.into_bytes())?;
                pre_tag_size = 0;
            } else {
                seg.set_pre_tag_size(pre_tag_size);
                if (tp == "video" || tp == "all") && seg.has_video_tag() {
                    ofp.write_all(&seg.into_bytes())?;
                    pre_tag_size = seg.tag().as_ref().unwrap().tag_size();
                } else if (tp == "audio" || tp == "all") && seg.has_audio_tag() {
                    ofp.write_all(&seg.into_bytes())?;
                    pre_tag_size = seg.tag().as_ref().unwrap().tag_size();
                } else if seg.has_script_tag() {
                    ofp.write_all(&seg.into_bytes())?;
                    pre_tag_size = seg.tag().as_ref().unwrap().tag_size();
                }
            }
        } else {
            break;
        }
    }
    return Ok(());
}

fn video_codec_name(id: &f64) -> String {
    let _id = *id as i32;
    return match _id {
        1 => "JPEG",
        2 => "H.263",
        3 => "Screen video",
        4 => "On2 VP6",
        5 => "On2 VP6 with alpha channel",
        6 => "Screen video version 2",
        7 => "AVC",
        _ => "Unknown",
    }
    .to_string();
}

fn audio_codec_name(id: &f64) -> String {
    let _id = *id as i32;
    return match _id {
        0 => "Linear PCM, platform endian",
        1 => "ADPCM",
        2 => "MP3",
        3 => "Linear PCM, little endian",
        4 => "Nellymoser 16-kHz mono",
        5 => "Nellymoser 8-kHz mono",
        6 => "Nellymoser",
        7 => "G.711 A-law logarithmic PCM",
        8 => "G.711 mu-law logarithmic PCM",
        9 => "reserved",
        10 => "AAC",
        11 => "Speex",
        14 => "MP3 8-Khz",
        15 => "Device-specific sound",
        _ => "Unknown",
    }
    .to_string();
}

fn show_flv_info(file: &str) -> io::Result<()> {
    let mut fp = File::open(file)?;
    let header = Header::from_reader(&mut fp)?;
    let mut script: Option<Segment> = Option::None;
    loop {
        match Segment::from_reader(&mut fp) {
            Ok(segment) => {
                if segment.has_script_tag() {
                    script.replace(segment);
                    break;
                }
            }
            Err(e) => {
                println!("Decode tag failed:{}", e);
                break;
            }
        }
    }
    println!("flv version :{}", header.version());
    println!("video: {}", if header.has_video() { "√" } else { "×" });
    println!("audio: {}", if header.has_audio() { "√" } else { "×" });
    if script.is_some() {
        let output_fields = [
            "duration".to_string(),
            "width".to_string(),
            "height".to_string(),
            "framerate".to_string(),
            "videocodecid".to_string(),
            "audiocodecid".to_string(),
        ];
        let tag_seg = script.unwrap();
        let tag_data = tag_seg.tag().as_ref().unwrap().data();
        if let TagData::Script(values) = tag_data {
            for value in values {
                if let Value::EcmaArray { entries } = value {
                    for kv in entries {
                        if output_fields.contains(&kv.key) {
                            if kv.key == "videocodecid".to_string() {
                                print!("video codec:");
                            } else if kv.key == "audiocodecid".to_string() {
                                print!("audio codec:")
                            } else {
                                print!("{}: ", &kv.key);
                            }
                            match &kv.value {
                                Value::String(s) => {
                                    println!("{}", s);
                                }
                                Value::Number(n) => {
                                    if kv.key == "videocodecid".to_string() {
                                        println!("{}", video_codec_name(n));
                                    } else if kv.key == "audiocodecid".to_string() {
                                        println!("{}", audio_codec_name(n));
                                    } else {
                                        println!("{:.0}", n);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
    return Ok(());
}
