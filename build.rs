use std::io::Write;

use anyhow::Result;
use bzip2::{write::BzEncoder, Compression};

fn main() -> Result<()> {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icon.ico");
        res.compile()?;
    }

    let icon_rgba = image::open("icon.png")?.to_rgba8().into_raw();

    let mut output = vec![];
    let mut zip = BzEncoder::new(&mut output, Compression::best());
    zip.write_all(&icon_rgba)?;
    zip.finish()?;

    let mut zip = std::fs::File::create("icon.rgba.bzip2")?;
    zip.write_all(&output)?;
    zip.flush()?;

    Ok(())
}
