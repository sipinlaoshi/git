use anyhow::bail;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;

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
        hash_object: String,
    },
}

enum Kind {
    Blob,
}

fn main() -> Result<()> {
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
            pretty_print,
            hash_object,
        } => {
            let path = format!(".git/objects/{}/{}", &hash_object[..2], &hash_object[2..]);
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
    }

    Ok(())
}
