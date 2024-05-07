use anyhow::bail;
use anyhow::Context;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::ffi::CStr;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    CatFile {
        #[arg(short)]
        pretty_print: bool,
        object: String,
    },
    HashObject {
        #[arg(short)]
        write: bool,
        file: PathBuf,
    },
}

enum Kind {
    Blob,
}

struct HashWriter<W> {
    hasher: Sha1,
    w: W,
}

impl<W> Write for HashWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.w.write(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.w.flush()
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory")
        }

        Commands::CatFile {
            pretty_print: _,
            object,
        } => {
            let path = format!(".git/objects/{}/{}", &object[..2], &object[2..]);
            let file = fs::File::open(path).context("open {path} fail")?;
            let zlib = ZlibDecoder::new(file);
            let mut zlib = BufReader::new(zlib);
            let mut buf = Vec::new();
            zlib.read_until(0, &mut buf).context("read header")?;
            let header =
                CStr::from_bytes_with_nul(&buf[..]).expect("header is konw to be ended by 0");
            let header = header.to_str().expect("header is not utf-8");
            let Some((kind, size)) = header.split_once(' ') else {
                bail!("header {header} is not split by space");
            };

            let kind = match kind {
                "blob" => Kind::Blob,
                _ => {
                    bail!("we don't kown how to print {kind}")
                }
            };

            let size = size
                .parse::<usize>()
                .context("header size is not a usize number")?;
            buf.clear();
            buf.reserve(size);
            buf.resize(size, 0);
            match kind {
                Kind::Blob => {
                    let stdout = io::stdout();
                    let mut stdout = stdout.lock();
                    io::copy(&mut zlib.take(size as u64), &mut stdout).context("read content")?;
                }
            }
        }

        Commands::HashObject { write, file } => {
            let mut file =
                fs::File::open(&file).with_context(|| format!("open file {}", file.display()))?;
            let stat = fs::File::metadata(&file).context("file stat")?;
            let size = stat.size();
            fn write_aux<W: Write>(
                mut w: HashWriter<W>,
                size: u64,
                file: &mut fs::File,
            ) -> anyhow::Result<String> {
                write!(w, "blob {}\0", size)?;
                //stream file to tmp
                std::io::copy(file, &mut w).context("copy file to tmp")?;
                let hash = w.hasher.finalize();
                Ok(hex::encode(hash))
            }

            let hash = if write {
                let tmp_path = "temporary";
                let tmp = fs::File::create(tmp_path).context("create temp file")?;
                let e = ZlibEncoder::new(tmp, Compression::default());
                let hash_writer = HashWriter {
                    hasher: Sha1::new(),
                    w: e,
                };
                let hash = write_aux(hash_writer, size, &mut file)?;
                std::fs::create_dir_all(format!(".git/objects/{}/", &hash[..2]))
                    .context("create new dir for blob")?;
                std::fs::rename(
                    tmp_path,
                    format!(".git/objects/{}/{}", &hash[..2], &hash[2..]),
                )
                .with_context(|| format!("move tmp file {} to real object file", tmp_path))?;
                hash
            } else {
                let hash = write_aux(
                    HashWriter {
                        hasher: Sha1::new(),
                        w: std::io::sink(),
                    },
                    size,
                    &mut file,
                )?;
                hash
            };

            println!("{hash}");
        }
    }

    anyhow::Ok(())
}
