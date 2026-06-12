use std::fs;
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(name = "exif_rm", version, about = "Remove metadata from files")]
struct Args {
    /// Files to process
    files: Vec<PathBuf>,

    /// Output directory (default: overwrite in-place)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Create backup files with given extension before modifying
    #[arg(long)]
    backup: Option<String>,

    /// Remove ICC color profiles (default: keep)
    #[arg(long)]
    strip_icc: bool,

    /// Suppress non-error output
    #[arg(short, long)]
    quiet: bool,
}

fn main() {
    let args = Args::parse();

    if args.files.is_empty() {
        eprintln!("Error: no files specified. Use --help for usage.");
        std::process::exit(1);
    }

    let options = exif_rm::RemovalOptions {
        icc_profile: args.strip_icc,
        ..exif_rm::RemovalOptions::default()
    };

    let mut had_error = false;

    for path in &args.files {
        match process_file(path, &args, &options) {
            Ok(()) => {
                if !args.quiet {
                    println!("{}", path.display());
                }
            }
            Err(e) => {
                eprintln!("{}: {e}", path.display());
                had_error = true;
            }
        }
    }

    if had_error {
        std::process::exit(1);
    }
}

fn process_file(path: &PathBuf, args: &Args, options: &exif_rm::RemovalOptions) -> exif_rm::Result<()> {
    let input = fs::read(path)?;

    let output = exif_rm::strip_metadata_with(&input, options)?;

    if let Some(ref backup_ext) = args.backup {
        let backup_path = {
            let mut p = path.clone();
            let mut new_name = p.file_name().unwrap().to_os_string();
            new_name.push(".");
            new_name.push(backup_ext);
            p.set_file_name(new_name);
            p
        };
        fs::copy(path, &backup_path)?;
    }

    if let Some(ref out_dir) = args.output {
        fs::create_dir_all(out_dir)?;
        let out_path = out_dir.join(path.file_name().unwrap());
        fs::write(&out_path, &output)?;
    } else {
        fs::write(path, &output)?;
    }

    Ok(())
}
