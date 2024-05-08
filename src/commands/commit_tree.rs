use std::fmt::Write;

use anyhow::Context;

use crate::object::{Kind, Object};

pub fn commit_tree_for(
    parent: Option<String>,
    message: String,
    tree_hash: String,
) -> anyhow::Result<[u8; 20]> {
    let mut content = String::new();
    writeln!(content, "tree {tree_hash}")?;
    _ = parent.inspect(|parent_hash| writeln!(content, "parent {parent_hash}").expect(""));
    writeln!(content, "author Xiezk <you@example.com> 1715157556 +0000")?;
    writeln!(
        content,
        "committer Xiezk <you@example.com> 1715157556 +0000"
    )?;
    writeln!(content)?;
    writeln!(content, "{message}")?;

    let mut commit = Object {
        kind: Kind::Commit,
        size: content.len(),
        reader: std::io::Cursor::new(content),
    };

    commit.write_to_objects().context("write commit object")
}

pub fn invoke(parent: Option<String>, message: String, tree_hash: String) -> anyhow::Result<()> {
    let hash = commit_tree_for(parent, message, tree_hash).context("commit tree")?;
    println!("{}", hex::encode(hash));
    Ok(())
}
