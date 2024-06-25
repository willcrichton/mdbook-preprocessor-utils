use std::{ops::Range, path::Path};

use clap::Parser;
use mdbook_preprocessor_utils::SimplePreprocessor;

#[derive(Parser)]
#[clap(author, about, version)]
struct ExampleArgs;

struct ExamplePreprocessor;

impl SimplePreprocessor for ExamplePreprocessor {
  type Args = ExampleArgs;

  fn name() -> &'static str {
    "example"
  }

  fn build(_ctx: &mdbook::preprocess::PreprocessorContext) -> mdbook::errors::Result<Self> {
    Ok(ExamplePreprocessor)
  }

  fn replacements(
    &self,
    _chapter_dir: &Path,
    _content: &str,
  ) -> mdbook::errors::Result<Vec<(Range<usize>, String)>> {
    Ok(Vec::new())
  }

  fn linked_assets(&self) -> Vec<mdbook_preprocessor_utils::Asset> {
    Vec::new()
  }

  fn all_assets(&self) -> Vec<mdbook_preprocessor_utils::Asset> {
    Vec::new()
  }
}

fn main() {
  mdbook_preprocessor_utils::main::<ExamplePreprocessor>()
}
