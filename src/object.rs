use anyhow::bail;
use anyhow::Context;
use anyhow::Ok;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::Digest;
use sha1::Sha1;
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
    pub fn read(object: &str) -> anyhow::Result<Object<impl BufRead>> {
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

struct HashWriter<W> {
    hasher: Sha1,
    writer: W,
}

impl<W: Write> Write for HashWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.writer.write(buf)?;
        self.hasher.update(&buf[..n]);
        std::result::Result::Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl<R: Read> Object<R> {
    pub fn write(&mut self, writer: impl Write) -> anyhow::Result<[u8; 20]> {
        let e = ZlibEncoder::new(writer, Compression::default());
        let mut hash_writer = HashWriter {
            hasher: Sha1::new(),
            writer: e,
        };
        write!(hash_writer, "{} {}\0", self.kind.to_string(), self.size)
            .context("write kind&&size")?;
        std::io::copy(&mut self.reader, &mut hash_writer).context("write content")?;
        hash_writer.flush().context("flush")?;
        let hash = hash_writer.hasher.finalize();
        Ok(hash.into())
    }

    pub fn write_to_objects(&mut self) -> anyhow::Result<[u8; 20]> {
        let tmp_path = "temp";
        let tmp_file =
            std::fs::File::create(tmp_path).context("create temp file for write object")?;
        let hash = self.write(&tmp_file).context("write to temp file")?;
        let encode_hash = hex::encode(hash);
        std::fs::create_dir_all(format!(".git/objects/{}/", &encode_hash[..2]))
            .context("create dir")?;
        std::fs::rename(
            tmp_path,
            format!(".git/objects/{}/{}", &encode_hash[..2], &encode_hash[2..]),
        )
        .context("move temp file to real object file")?;
        Ok(hash)
    }
}
