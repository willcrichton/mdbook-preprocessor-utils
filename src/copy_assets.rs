use anyhow::{Error, Result};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

pub fn copy_assets(src_dir: &Path, dst_dir: &Path) -> Result<()> {
  println!("cargo:rerun-if-changed={}", src_dir.display());

  let dst_dir = Path::new(dst_dir);
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
