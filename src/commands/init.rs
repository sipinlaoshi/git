use std::fs;

use anyhow::{Context, Ok};

pub fn invoke() -> anyhow::Result<()> {
    fs::create_dir(".git").context("create .git")?;
    fs::create_dir(".git/objects").context("create .git/objects")?;
    fs::create_dir(".git/refs").context("create .git/refs")?;
    fs::write(".git/HEAD", "ref: refs/heads/main\n").context("write .git/HEAD")?;
    println!("Initialized git directory");
    Ok(())
}
