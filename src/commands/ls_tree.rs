use crate::object::Object;
use anyhow::{Context, Ok};
use std::io::{BufRead, Read};

pub fn invoke(name_only: bool, tree_hash: &str) -> anyhow::Result<()> {
    let tree =
        Object::read_object(tree_hash).with_context(|| format!("read object {tree_hash}"))?;
    let mut reader = tree.reader;
    let mut buf = Vec::new();
    let mut entry_hash: [u8; 20] = [0; 20];
    loop {
        buf.clear();
        let n = reader.read_until(0, &mut buf).context("read mode&&name")?;
        if n == 0 {
            break;
        }
        let mode_name =
            std::ffi::CStr::from_bytes_with_nul(&buf).context("parse mode&&name to cstr")?;
        let mode_name = mode_name
            .to_str()
            .context("mode&&name is not valid utf-8")?;
        let (mode, name) = mode_name
            .split_once(' ')
            .context("mode&&name is not split by space")?;
        reader
            .read_exact(&mut entry_hash)
            .context("read 20 bytes of entry hash")?;
        if name_only {
            println!("{}", name);
        } else {
            let entry_hash = hex::encode(entry_hash);
            let entry_obj = Object::read_object(&entry_hash).context("read entry object")?;
            let kind = entry_obj.kind.to_string();
            println!("{:0>6} {} {}    {}", mode, kind, entry_hash, name);
        }
    }

    Ok(())
}
