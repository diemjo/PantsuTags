use std::error::Error;
use std::fs;
use std::path::Path;
use clap::{IntoApp};
use clap_generate::generators::{Bash, generate_to};
include!("src/cli.rs");

fn main() -> Result<(), Box<dyn Error>>{
    let mut app = Args::into_app();
    let outdir = Path::new(env!("CARGO_MANIFEST_DIR")).join("completions/");
    fs::create_dir_all(&outdir)?;
    generate_to(Bash, &mut app, "pantsu-tags", outdir)?;
    Ok(())
}