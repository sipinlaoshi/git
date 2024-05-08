use std::io::Read;

use anyhow::bail;
use anyhow::ensure;
use anyhow::Context;

use crate::object::Kind;
use crate::object::Object;

pub fn invoke(pretty_print: bool, object: &str) -> anyhow::Result<()> {
    ensure!(pretty_print, "we only support -p");
    let mut buf = Vec::new();
    let obj = Object::read_object(object).context("read object")?;

    buf.reserve(obj.size);
    buf.resize(obj.size, 0);
    match obj.kind {
        Kind::Blob => {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            std::io::copy(&mut obj.reader.take(obj.size as u64), &mut stdout)
                .context("read content")
                .map(|_| ())
        }
        _ => bail!("we don't konw how to print a {}", obj.kind),
    }
}
