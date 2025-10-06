use anyhow::{Result, anyhow};
use clap::Parser;
use rexiv2::Metadata;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
/// A tool for tagging Exif metadata to scanned images from film rolls.
struct Args {
    /// Source directory of files to apply metadata to.
    src: Vec<PathBuf>,

    /// Set the ISO film speed used.
    #[arg(short, long)]
    iso: Option<u16>,

    /// Set the camera model used.
    /// First word is parsed as the camera maker while the rest is set as the camera model.
    #[arg(short, long)]
    camera: Option<String>,
}

fn safe_write_metadata(file: &PathBuf, meta: &Metadata) -> Result<()> {
    let temp = tempfile::NamedTempFile::new_in(file.parent().unwrap())?;
    meta.save_to_file(&temp.path())?;
    temp.persist(&file)?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.src.is_empty() {
        return Err(anyhow!("No files were provided"));
    }
    if args.iso.is_none() && args.camera.is_none() {
        return Err(anyhow!("No flags for modifying the metadata were provided"));
    }

    for file in &args.src {
        let meta = Metadata::new_from_path(file)?;

        if let Some(iso) = args.iso {
            meta.set_tag_numeric("Exif.Photo.ISOSpeedRatings", i32::from(iso))?;
        }

        if let Some(camera) = &args.camera {
            let (make, model) = camera.split_once(' ').unwrap_or_default();
            meta.set_tag_string("Exif.Image.Make", make)?;
            meta.set_tag_string("Exif.Image.Model", model)?;
        }

        safe_write_metadata(&file, &meta)?;
    }

    Ok(())
}
