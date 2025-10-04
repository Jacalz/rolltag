use anyhow::{Result, anyhow};
use clap::Parser;
use rexiv2::Metadata;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
/// A tool for tagging Exif metadata to scanned images from film rolls.
struct Args {
    /// Source directory of files to apply metadata to.
    src: PathBuf,

    /// Set the ISO film speed used.
    #[arg(short, long)]
    iso: Option<u16>,

    /// Set the camera model used.
    /// First word is parsed as the camera maker while the rest is set as the camera model.
    #[arg(short, long)]
    camera: Option<String>,

    /// Set the lens model used.
    #[arg(short, long)]
    lens: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if !args.src.exists() {
        return Err(anyhow!("Source directory does not exist"));
    }

    let meta = Metadata::new_from_path(&args.src)?;
    println!("{:?}", meta.get_tag_multiple_strings("Exif.Image.Make")?);

    Ok(())
}
