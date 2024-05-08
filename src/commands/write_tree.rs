use std::{
    cmp::Ordering,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use anyhow::{Context, Ok};

use crate::object::{Kind, Object};

pub fn write_tree_for(path: &Path) -> anyhow::Result<Option<[u8; 20]>> {
    let dir_entries =
        std::fs::read_dir(path).with_context(|| format!("read directory {}", path.display()))?;
    let mut entries = Vec::new();
    for entry in dir_entries {
        let entry = entry.context("not a entry")?;
        let name = entry.file_name();
        let meta = entry
            .metadata()
            .with_context(|| format!("get stat of {}", entry.path().display()))?;
        entries.push((entry, name, meta));
    }
    entries.sort_unstable_by(|a, b| {
        let an = a.1.as_encoded_bytes();
        let bn = b.1.as_encoded_bytes();
        let len = std::cmp::min(an.len(), bn.len());
        match an[..len].cmp(&bn[..len]) {
            Ordering::Equal => {}
            o => return o,
        }

        if an.len() == bn.len() {
            return Ordering::Equal;
        }

        let c1 = if let Some(c) = an.get(len).copied() {
            Some(c)
        } else if a.2.is_dir() {
            Some(b'/')
        } else {
            None
        };

        let c2 = if let Some(c) = bn.get(len).copied() {
            Some(c)
        } else if b.2.is_dir() {
            Some(b'/')
        } else {
            None
        };

        c1.cmp(&c2)
    });

    let mut contents: Vec<u8> = Vec::new();

    for (entry, name, meta) in entries {
        let path = entry.path();
        if name == ".git" || name == "target" {
            continue;
        }
        let name = name.as_encoded_bytes();
        let mode = if meta.is_dir() {
            "40000"
        } else if meta.is_file() {
            let mode = meta.mode();
            if mode & 0o111 != 0 {
                "100755"
            } else {
                "100644"
            }
        } else {
            "120000"
        };

        contents.extend(mode.as_bytes());
        contents.push(b' ');
        contents.extend(name);
        contents.push(0);

        if meta.is_dir() {
            _ = write_tree_for(&path)
                .context("recursive write sub dir")?
                .inspect(|hash| contents.extend(hash));
        } else {
            let mut blob = Object {
                kind: Kind::Blob,
                size: meta.size() as usize,
                reader: std::fs::File::open(&path).context("create blob")?,
            };
            blob.write_to_objects()
                .inspect(|hash| contents.extend(hash))
                .context("write blob to objects file")?;
        }
    }
    if contents.is_empty() {
        return Ok(None);
    }

    let mut tree = Object {
        kind: Kind::Tree,
        size: contents.len(),
        reader: std::io::Cursor::new(contents),
    };

    let hash = tree
        .write_to_objects()
        .context("write tree to object file")?;
    Ok(Some(hash))
}

pub fn invoke() -> anyhow::Result<()> {
    _ = write_tree_for(&PathBuf::from("."))
        .context("write tree")?
        .inspect(|hash| {
            println!("{}", hex::encode(hash));
        });
    Ok(())
}
