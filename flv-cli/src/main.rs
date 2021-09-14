use clap::{App, Arg, SubCommand};
use flv_parser::flv::{Header, ParseResult, Parser, ScriptTagDataTrait, Segment, TagData};
use std::{
    fs::File,
    io::{self, BufWriter, Read, Write},
    vec,
};

fn main() {
    let matches = App::new("flv cli")
        .version("0.1")
        .author("Shell asypost@gmail.com")
        .arg(
            Arg::with_name("FILE")
                .help("flv file,- for pipe")
                .required(true),
        )
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
                if let Err(e) = extract(file, tp, out) {
                    if e.kind() != io::ErrorKind::BrokenPipe {
                        eprintln!("Error: {}", e);
                    }
                }
            }
        }
    } else {
        println!("{}", matches.usage());
    }
}

fn extract(src: &str, tp: &str, path: &str) -> io::Result<()> {
    let stdin = io::stdin();
    let mut fp: Box<dyn Read> = if src == "-" {
        Box::new(stdin.lock())
    } else {
        Box::new(File::open(src)?)
    };
    let stdout = io::stdout();
    let mut ofp: BufWriter<Box<dyn io::Write>> = if path == "-" {
        BufWriter::with_capacity(4 * 1024, Box::new(stdout.lock()))
    } else {
        BufWriter::with_capacity(4 * 1024, Box::new(File::create(path)?))
    };
    let mut parser = Parser::new();
    let mut buffer: Vec<u8> = vec![0x00; 100 * 1024];
    loop {
        let count = fp.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        parser.feed(&buffer[..count]);
        loop {
            match parser.parse()? {
                ParseResult::MoreDataRequired(_bytes) => {
                    break;
                }
                ParseResult::Header(mut header) => {
                    header.set_has_video((tp == "all" || tp == "video") && header.has_video());
                    header.set_has_audio((tp == "all" || tp == "audio") && header.has_audio());
                    ofp.write_all(&header.into_bytes())?;
                    ofp.write_all(&0_u32.to_be_bytes())?;
                }
                ParseResult::PreTagSize(_) => {}
                ParseResult::Tag(tag) => {
                    if (tp == "video" || tp == "all") && tag.is_video_tag() {
                        ofp.write_all(&tag.into_bytes())?;
                        ofp.write_all(&tag.tag_size().to_be_bytes())?;
                    } else if (tp == "audio" || tp == "all") && tag.is_audio_tag() {
                        ofp.write_all(&tag.into_bytes())?;
                        ofp.write_all(&tag.tag_size().to_be_bytes())?;
                    } else if tag.is_script_tag() {
                        ofp.write_all(&tag.into_bytes())?;
                        ofp.write_all(&tag.tag_size().to_be_bytes())?;
                    }
                }
            }
        }
    }
    ofp.flush()?;
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
    let stdin = io::stdin();
    let mut fp: Box<dyn Read> = if file == "-" {
        Box::new(stdin.lock())
    } else {
        Box::new(File::open(file)?)
    };
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
    println!("version: {}", header.version());
    println!("video: {}", if header.has_video() { "yes" } else { "no" });
    println!("audio: {}", if header.has_audio() { "yes" } else { "no" });
    if script.is_some() {
        let tag_seg = script.unwrap();
        let tag_data = tag_seg.tag().as_ref().unwrap().data();
        if let TagData::Script(values) = tag_data {
            println!("duration: {:0.3}s", values.duration());
            println!("width : {:0.0}", values.width());
            println!("height: {:0.0}", values.height());
            println!("fps: {:0.0}", values.framerate());
            println!(
                "video codec: {}",
                video_codec_name(&values.video_codec_id())
            );
            println!(
                "audio codec: {}",
                audio_codec_name(&values.audio_codec_id())
            );
        }
    }
    return Ok(());
}
