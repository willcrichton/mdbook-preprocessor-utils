use anyhow::{Error, Result};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

pub fn copy_assets(src_dir: impl AsRef<Path>, dst_dir: impl AsRef<Path>) -> Result<()> {
  let src_dir = src_dir.as_ref();
  let dst_dir = dst_dir.as_ref();

  println!("cargo:rerun-if-changed={}", src_dir.display());
  fs::create_dir_all(dst_dir)?;

  let src_entries = match fs::read_dir(src_dir) {
    Ok(src_entries) => src_entries,
    Err(err) => match err.kind() {
      ErrorKind::NotFound => return Ok(()),
      _ => return Err(Error::new(err)),
    },
  };

  for entry in src_entries {
    let path = entry?.path();
    fs::copy(&path, dst_dir.join(path.file_name().unwrap()))?;
  }

  Ok(())
}
