use anyhow::{Context, Result};
use clap::Parser;
use keyball44_viz::{generate_svg, is_empty_key, parse_layers, Layer};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the keymap.c file
    keymap_file: PathBuf,

    /// Display statistics about the keymap
    #[arg(short, long, default_value_t = false)]
    show_stats: bool,

    /// Output SVG file name
    #[arg(short, long)]
    output_file: Option<PathBuf>,
}

fn print_stats(layers: &[Layer]) {
    for (i, layer) in layers.iter().enumerate() {
        let total_keys = layer.keys.iter().flatten().count();
        let assigned_keys = layer
            .keys
            .iter()
            .flatten()
            .filter(|key| !is_empty_key(key))
            .count();
        let unassigned_keys = total_keys - assigned_keys;
        println!(
            "Layer {}: Total Keys: {}, Assigned Keys: {}, Unassigned Keys: {}",
            i, total_keys, assigned_keys, unassigned_keys
        );
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let content = fs::read_to_string(&args.keymap_file).context(format!(
        "Failed to read keymap file: {:?}",
        args.keymap_file
    ))?;

    let layers = parse_layers(&content)?;

    if args.show_stats {
        print_stats(&layers);
    }

    let svg = generate_svg(&layers);

    // Write SVG to the specified output file or default to keymap filename
    if let Some(output_file) = args.output_file {
        fs::write(output_file, svg).context("Failed to write SVG file")?;
    } else {
        let basename = args
            .keymap_file
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or(anyhow::anyhow!("Invalid filename"))
            .context("Unable to retrieve output file basename")?;
        let output_path = PathBuf::from(format!("{}.svg", basename));
        fs::write(output_path, svg).context("Failed to write SVG file")?;
    }

    Ok(())
}
