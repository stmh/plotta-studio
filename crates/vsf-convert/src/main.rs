//! vsf-convert: Convert single-line fonts to VSF format
//!
//! A CLI tool for converting vintage and procedural single-line fonts
//! to the VSF (Vector Stroke Font) JSON format.

mod formats;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use drawing_text::VsfFont;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "vsf-convert")]
#[command(about = "Convert single-line fonts to VSF format")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert Asteroids arcade font (1979 Atari)
    Asteroids {
        /// Input JSON file with font data (omit to use embedded data)
        input: Option<PathBuf>,
        /// Output VSF file path
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Convert Apple 410 plotter font (1983)
    Apple410 {
        /// Input JSON file with font data (omit to use embedded data)
        input: Option<PathBuf>,
        /// Output VSF file path
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Convert minf ultra-minimal font (base64 encoded)
    Minf {
        /// Base64 encoded font data (omit to use embedded data)
        input: Option<String>,
        /// Output VSF file path
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Convert all embedded fonts to VSF
    All {
        /// Output directory for VSF files
        #[arg(short, long, default_value = "fonts/vsf")]
        output_dir: PathBuf,
    },
    /// List available font formats
    List,
}

/// Convert a font from input (file or embedded) and write to output
fn convert_font<F>(
    name: &str,
    input: Option<PathBuf>,
    output: &PathBuf,
    embedded: &str,
    parse: F,
) -> Result<()>
where
    F: FnOnce(&str) -> Result<VsfFont>,
{
    println!("Converting {} font...", name);
    let data = match input {
        Some(path) => std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?,
        None => embedded.to_string(),
    };
    let font = parse(&data)?;
    write_vsf(&font, output)?;
    println!("Written to {}", output.display());
    Ok(())
}

/// Convert and write a font using embedded data
fn convert_embedded<F>(output_dir: &PathBuf, filename: &str, embedded: &str, parse: F) -> Result<()>
where
    F: FnOnce(&str) -> Result<VsfFont>,
{
    let font = parse(embedded)?;
    let path = output_dir.join(filename);
    write_vsf(&font, &path)?;
    println!("  Written {}", path.display());
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Asteroids { input, output } => {
            convert_font("Asteroids", input, &output, formats::asteroids::embedded_data(), formats::asteroids::parse)?;
        }
        Commands::Apple410 { input, output } => {
            convert_font("Apple 410", input, &output, formats::apple410::embedded_data(), formats::apple410::parse)?;
        }
        Commands::Minf { input, output } => {
            println!("Converting minf font...");
            let data = match input {
                Some(s) if s.starts_with('@') => std::fs::read_to_string(&s[1..])?,
                Some(s) => s,
                None => formats::minf::EMBEDDED_DATA.to_string(),
            };
            let font = formats::minf::parse(&data)?;
            write_vsf(&font, &output)?;
            println!("Written to {}", output.display());
        }
        Commands::All { output_dir } => {
            std::fs::create_dir_all(&output_dir)
                .with_context(|| format!("Failed to create {}", output_dir.display()))?;

            println!("Converting all embedded fonts to {}...", output_dir.display());

            convert_embedded(&output_dir, "asteroids.vsf", formats::asteroids::embedded_data(), formats::asteroids::parse)?;
            convert_embedded(&output_dir, "apple410.vsf", formats::apple410::embedded_data(), formats::apple410::parse)?;
            convert_embedded(&output_dir, "minf.vsf", formats::minf::EMBEDDED_DATA, formats::minf::parse)?;

            println!("Done!");
        }
        Commands::List => {
            println!("Available font formats:");
            println!();
            println!("  asteroids  - Asteroids arcade font (1979 Atari)");
            println!("               Source: JS/JSON coordinate arrays");
            println!("               Characters: A-Z, 0-9, punctuation");
            println!();
            println!("  apple410   - Apple 410 Color Plotter font (1983)");
            println!("               Source: JSON with stroke coordinates");
            println!("               Characters: Full ASCII printable set");
            println!();
            println!("  minf       - Ultra-minimal procedural font (2024)");
            println!("               Source: 72-byte base64 string");
            println!("               Characters: a-z, A-Z");
            println!();
            println!("Use 'vsf-convert all' to convert all embedded fonts at once.");
        }
    }

    Ok(())
}

fn write_vsf(font: &VsfFont, path: &PathBuf) -> Result<()> {
    let json = font.to_json()?;
    std::fs::write(path, json).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}
