use anyhow::{Context, Ok};

pub fn invoke(repo: String, dir: Option<String>) -> anyhow::Result<()> {
    let ref_url =
        format!("{repo}https://github.com/llvm/llvm-project.git/info/refs?service=git-upload-pack");
    let resp = reqwest::blocking::get(&ref_url).with_context(|| format!("get {ref_url}"))?;
    //let json = resp.json()?;
    println!("{resp:?}");
    Ok(())
}
