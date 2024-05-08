use anyhow::bail;
use anyhow::Context;
use anyhow::Ok;
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;

#[derive(PartialEq, Eq, Debug)]
pub(crate) enum Kind {
    Blob,
    Tree,
    Commit,
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Kind::*;
        match self {
            Blob => write!(f, "blob"),
            Tree => write!(f, "tree"),
            Commit => write!(f, "commit"),
        }
    }
}

impl Kind {
    pub fn new(kind: &str) -> Option<Kind> {
        match kind {
            "blob" => Some(Kind::Blob),
            "tree" => Some(Kind::Tree),
            "commit" => Some(Kind::Commit),
            _ => None,
        }
    }
}

pub(crate) struct Object<R> {
    pub(crate) kind: Kind,
    pub(crate) size: usize,
    pub(crate) reader: R,
}

impl Object<()> {
    pub fn read_object(object: &str) -> anyhow::Result<Object<impl BufRead>> {
        let path = format!(".git/objects/{}/{}", &object[..2], &object[2..]);
        let file = fs::File::open(path).context("open {path} fail")?;
        let zlib = ZlibDecoder::new(file);
        let mut zlib = BufReader::new(zlib);
        let mut buf = Vec::new();
        zlib.read_until(0, &mut buf).context("read header")?;
        let header = CStr::from_bytes_with_nul(&buf[..]).expect("header is konw to be ended by 0");
        let header = header.to_str().expect("header is not utf-8");
        let Some((kind, size)) = header.split_once(' ') else {
            bail!("header {header} is not split by space");
        };

        let kind = Kind::new(kind).with_context(|| format!("unkown object kind {}", kind))?;

        let size = size
            .parse::<usize>()
            .context("header size is not a usize number")?;

        Ok(Object {
            kind,
            size,
            reader: zlib,
        })
    }
}
