use anyhow::{Result, anyhow};
use clap::Parser;
use rayon::ThreadPoolBuilder;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rexiv2::Metadata;
use std::fs;
use std::path::{Path, PathBuf};
use time::{OffsetDateTime, macros::format_description};

const DATE_TIME_FORMAT: &[time::format_description::FormatItem<'_>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
/// A tool for tagging Exif metadata to scanned images from film rolls.
struct Args {
    /// Source files to apply metadata to.
    src: Vec<PathBuf>,

    /// Set the film stock used.
    #[arg(short, long)]
    film: Option<String>,

    /// Set the ISO film speed used.
    #[arg(short, long)]
    iso: Option<u16>,

    /// Set the camera model used.
    /// First word is parsed as the camera maker while the rest is set as the camera model.
    #[arg(short, long)]
    camera: Option<String>,

    /// Set the lens model used.
    /// First word is parsed as the lens maker while the rest is set as the lens model.
    #[arg(short, long)]
    lens: Option<String>,

    /// Clear all metadata from the image before applying new metadata.
    #[arg(short, long)]
    clear: bool,

    /// Set the artist name.
    #[arg(short, long)]
    artist: Option<String>,

    /// Set the focal length of the lens used.
    #[arg(short, long)]
    focal_length: Option<u16>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.src.is_empty() {
        return Err(anyhow!("No files were provided"));
    }
    if args.iso.is_none() && args.camera.is_none() {
        return Err(anyhow!("No flags for modifying the metadata were provided"));
    }

    ThreadPoolBuilder::new().build()?.install(|| -> Result<()> {
        args.src
            .par_iter()
            .try_for_each(|path| -> Result<()> { apply_metadata(&args, path) })
    })
}

fn apply_metadata(args: &Args, file: &PathBuf) -> Result<()> {
    let meta = Metadata::new_from_path(file)?;

    if args.clear {
        meta.clear_exif();
    }

    set_timestamps(file, &meta)?;

    if let Some(film) = &args.film {
        meta.set_tag_string("Exif.Image.ImageDescription", film)?;
    }

    if let Some(iso) = args.iso {
        meta.set_tag_numeric("Exif.Photo.ISOSpeedRatings", i32::from(iso))?;
    }

    if let Some(camera) = &args.camera {
        let (make, model) = camera.split_once(' ').unwrap_or_default();
        meta.set_tag_string("Exif.Image.Make", make)?;
        meta.set_tag_string("Exif.Image.Model", model)?;
    }

    if let Some(focal_length) = args.focal_length {
        meta.set_tag_numeric("Exif.Image.FocalLength", i32::from(focal_length))?;
    }

    if let Some(lens) = &args.lens {
        let (make, model) = lens.split_once(' ').unwrap_or_default();
        meta.set_tag_string("Exif.Photo.LensMake", make)?;
        meta.set_tag_string("Exif.Photo.LensModel", model)?;
    }

    if let Some(artist) = &args.artist {
        meta.set_tag_string("Exif.Image.Artist", artist)?;
    }

    safe_write_metadata(file, &meta)
}

// This is required to ensure correct ordering when sorting files to avoid
// using the modification date as the primary sorting key.
fn set_timestamps(file: &Path, meta: &Metadata) -> Result<()> {
    let time = OffsetDateTime::from(file.metadata()?.created()?);
    let time_str = time.format(DATE_TIME_FORMAT)?;
    meta.set_tag_string("Exif.Photo.DateTimeOriginal", &time_str)?;
    meta.set_tag_string("Exif.Photo.DateTimeDigitized", &time_str)?;
    Ok(())
}

fn safe_write_metadata(file: &PathBuf, meta: &Metadata) -> Result<()> {
    let temp = tempfile::NamedTempFile::new_in(file.parent().unwrap())?;
    fs::copy(file, &temp)?;
    meta.save_to_file(temp.path())?;
    temp.persist(file)?;
    Ok(())
}
