use anyhow::{Context, Ok};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

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
    let mut file =
        fs::File::open(&file).with_context(|| format!("open file {}", file.display()))?;
    let stat = fs::File::metadata(&file).context("file stat")?;
    let size = stat.size();
    fn write_aux<W: Write>(w: W, size: u64, file: &mut fs::File) -> anyhow::Result<String> {
        let mut hash_writer = HashWriter {
            w,
            hasher: Sha1::new(),
        };
        write!(hash_writer, "blob {}\0", size)?;
        //stream file to tmp
        std::io::copy(file, &mut hash_writer).context("copy file to tmp")?;
        hash_writer.flush()?;
        let hash = hash_writer.hasher.finalize();
        Ok(hex::encode(hash))
    }

    let hash = if write {
        let tmp_path = "temporary";
        let tmp = fs::File::create(tmp_path).context("create temp file")?;
        let e = ZlibEncoder::new(tmp, Compression::default());
        let hash = write_aux(e, size, &mut file)?;
        std::fs::create_dir_all(format!(".git/objects/{}/", &hash[..2]))
            .context("create new dir for blob")?;
        std::fs::rename(
            tmp_path,
            format!(".git/objects/{}/{}", &hash[..2], &hash[2..]),
        )
        .with_context(|| format!("move tmp file {} to real object file", tmp_path))?;
        hash
    } else {
        let hash = write_aux(std::io::sink(), size, &mut file)?;
        hash
    };

    println!("{hash}");
    Ok(())
}
