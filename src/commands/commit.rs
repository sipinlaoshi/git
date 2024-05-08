use std::fs;

use anyhow::{bail, Context, Ok};

use crate::commands::{commit_tree, write_tree};

pub fn invoke(message: String) -> anyhow::Result<()> {
    let tree_hash =
        write_tree::write_tree_for(&std::path::PathBuf::from(".")).context("write tree")?;
    let tree_hash = tree_hash.expect("no thing to commit");
    let head_ref = fs::read_to_string(".git/HEAD").context("read head path")?;
    let head_ref = head_ref.trim();
    let head_path = if let Some(head_ref) = head_ref.strip_prefix("ref: ") {
        format!(".git/{head_ref}")
    } else {
        bail!("we don't konw how to deal with detach node");
    };
    let head_hash =
        fs::read_to_string(&head_path).with_context(|| format!("read head hash {head_path}"))?;
    let commit_hash =
        commit_tree::commit_tree_for(Some(head_hash), message, hex::encode(tree_hash))
            .context("commit tree")?;
    let commit_hash = hex::encode(commit_hash);
    fs::write(&head_path, hex::encode(&commit_hash))
        .with_context(|| format!("write commit hash to {head_path}"))?;
    println!("HEAD is now at {commit_hash}");
    Ok(())
}
