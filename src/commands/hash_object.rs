use anyhow::{Context, Ok};
use sha1::{Digest, Sha1};
use std::fs;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

use crate::object::{Kind, Object};

struct HashWriter<W> {
    hasher: Sha1,
    w: W,
}

impl<W> Write for HashWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.w.write(buf)?;
        self.hasher.update(&buf[..n]);
        std::result::Result::Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.w.flush()
    }
}

pub fn invoke(write: bool, file: &PathBuf) -> anyhow::Result<()> {
    let file = fs::File::open(&file).with_context(|| format!("open file {}", file.display()))?;
    let stat = fs::File::metadata(&file).context("file stat")?;
    let size = stat.size();
    let mut obj = Object {
        kind: Kind::Blob,
        size: size as usize,
        reader: file,
    };

    let hash = if write {
        obj.write(std::io::sink()).context("sink write")?
    } else {
        obj.write_to_objects().context("write to object")?
    };
    println!("{}", hex::encode(hash));
    Ok(())
}
