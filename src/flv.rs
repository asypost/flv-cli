mod header;
mod segment;
mod tag;

use std::io::{self, Read};

pub use header::Header;
pub use segment::Segment;
pub use tag::{Tag, TagData};

pub struct Container {
    header: Header,
    body: Vec<Segment>,
}

impl Container {
    pub fn from_reader(reader: &mut impl Read) -> io::Result<Self> {
        let header = Header::from_reader(reader)?;
        let mut body = vec![];
        loop {
            if let Ok(seg) = Segment::from_reader(reader) {
                body.push(seg);
            } else {
                break;
            }
        }
        return Ok(Self { header, body });
    }
}
