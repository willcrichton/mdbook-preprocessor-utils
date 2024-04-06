use std::{
  env,
  io::{self, Write},
  process,
};

use chrono::Local;
use clap::{Parser, Subcommand};
use env_logger::Builder;
use log::LevelFilter;
use mdbook::{
  errors::Error,
  preprocess::{CmdPreprocessor, Preprocessor},
};
use semver::{Version, VersionReq};

mod copy_assets;
mod html;
mod processor;
#[cfg(feature = "testing")]
pub mod testing;

pub use copy_assets::copy_assets;
pub use html::HtmlElementBuilder;
pub use mdbook;
pub use processor::{Asset, SimplePreprocessor};
pub use rayon;

#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
  #[clap(subcommand)]
  command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
  Supports { renderer: String },
}

// This is copied verbatim from mdbook so the style is consistent.
// https://github.com/rust-lang/mdBook/blob/94e0a44e152d8d7c62620e83e0632160977b1dd5/src/main.rs#L97-L121
fn init_logger() {
  let mut builder = Builder::new();

  builder.format(|formatter, record| {
    writeln!(
      formatter,
      "{} [{}] ({}): {}",
      Local::now().format("%Y-%m-%d %H:%M:%S"),
      record.level(),
      record.target(),
      record.args()
    )
  });

  if let Ok(var) = env::var("RUST_LOG") {
    builder.parse_filters(&var);
  } else {
    // if no RUST_LOG provided, default to logging at the Info level
    builder.filter(None, LevelFilter::Info);
    // Filter extraneous html5ever not-implemented messages
    builder.filter(Some("html5ever"), LevelFilter::Error);
  }

  builder.init();
}

pub fn main<P: SimplePreprocessor>() {
  init_logger();

  let args = Args::parse();
  let preprocessor = processor::SimplePreprocessorDriver::<P>::new();

  if let Some(Command::Supports { renderer }) = args.command {
    handle_supports(&preprocessor, &renderer);
  } else if let Err(e) = handle_preprocessing(&preprocessor) {
    eprintln!("{}", e);
    process::exit(1);
  }
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
  let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

  let book_version = Version::parse(&ctx.mdbook_version)?;
  let version_req = VersionReq::parse(mdbook::MDBOOK_VERSION)?;

  if !version_req.matches(&book_version) {
    eprintln!(
      "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
      pre.name(),
      mdbook::MDBOOK_VERSION,
      ctx.mdbook_version
    );
  }

  let processed_book = pre.run(&ctx, book)?;
  serde_json::to_writer(io::stdout(), &processed_book)?;

  Ok(())
}

fn handle_supports(pre: &dyn Preprocessor, renderer: &str) -> ! {
  let supported = pre.supports_renderer(renderer);

  // Signal whether the renderer is supported by exiting with 1 or 0.
  if supported {
    process::exit(0);
  } else {
    process::exit(1);
  }
}
