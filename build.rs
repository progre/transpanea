use std::{
    fs::{self, File},
    io,
};

use ico::{IconDirEntry, IconImage};
use tempfile::NamedTempFile;
use winresource::WindowsResource;

fn create_ico() -> io::Result<NamedTempFile> {
    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
    let file = File::open("icons/512x512.png")?;
    let image = IconImage::read_png(file)?;
    icon_dir.add_entry(IconDirEntry::encode(&image)?);
    let file = File::open("icons/64x64.png")?;
    let image = IconImage::read_png(file)?;
    icon_dir.add_entry(IconDirEntry::encode(&image)?);
    let mut file = NamedTempFile::new()?;
    icon_dir.write(&mut file)?;
    Ok(file)
}

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = WindowsResource::new();
        let file = create_ico().unwrap();
        res.set_icon(file.path().to_string_lossy().as_ref());
        let result = res.compile();
        fs::remove_file(file.path().to_string_lossy().as_ref()).unwrap();
        result.unwrap();
    }
}
