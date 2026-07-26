use clap::Parser;
use rayon::prelude::*;
use sha2::{Digest as Sha2Digest, Sha256};
use std::{
    error::Error,
    ffi::OsStr,
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    result::Result,
};

#[derive(Parser)]
#[command(name = "hashname", version, about = "Rename files to their hash")]
struct Args {
    /// Files to process
    #[arg(num_args = 1..)]
    files: Vec<String>,

    /// Do not actually rename files
    #[arg(short, long)]
    dry_run: bool,
    /// Process the file even if its name looks like it has already been processed
    #[arg(short, long)]
    force_rehash: bool,
    /// Rename/replace a file even there if another file with the same name exists
    #[arg(short = 'F', long)]
    force_rename: bool,
    /// Explain what is being done
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let opts = Args::parse();

    opts.files
        .par_iter()
        .for_each(|x| match process_file(&opts, &x) {
            Ok(result) => {
                if opts.verbose && result.len() > 0 {
                    println!("\"{}\" -> \"{}\"", x, result);
                }
            }
            Err(msg) => {
                if opts.verbose {
                    eprintln!("Skipped \"{}\": {}", x, msg.to_string());
                }
            }
        });
}

fn process_file(opts: &Args, raw_filename: &String) -> Result<String, Box<dyn Error>> {
    let path_file = Path::new(raw_filename);
    if is_file(raw_filename)? {
        return Err(Box::from("Not a file"));
    }

    let file_name = get_str_from_osstr(&path_file.file_stem())?;
    if !opts.force_rehash && is_already_processed(&file_name) {
        return Err(Box::from("Already processed"));
    }

    let file_ext = get_str_from_osstr(&path_file.extension())?;
    let result_hash = calculate_file_sha256(path_file)?;
    let result_filename = match file_ext.len() {
        0 => result_hash,
        _ => format!("{}.{}", result_hash, file_ext),
    };
    let mut path = PathBuf::from(path_file);
    path.set_file_name(result_filename.clone());
    let result_path = match path.into_os_string().into_string() {
        Ok(s) => s,
        Err(_) => return Err(Box::from("Could not process filename")),
    };

    if !opts.force_rename && is_already_exists(&result_path) {
        return Err(Box::from("Already exists"));
    }

    if !opts.dry_run {
        fs::rename(raw_filename, result_path.clone())?;
    }

    Ok(result_path)
}

fn is_file(raw_filename: &String) -> Result<bool, Box<dyn Error>> {
    Ok(fs::symlink_metadata(raw_filename)?.file_type().is_symlink()
        || !fs::metadata(raw_filename)?.file_type().is_file())
}

fn get_str_from_osstr(osstr: &Option<&OsStr>) -> Result<String, Box<dyn Error>> {
    Ok(match osstr {
        Some(n) => match n.to_str() {
            Some(s) => s.to_string(),
            None => return Err(Box::from("Could not get string from OsStr")),
        },
        None => return Err(Box::from("Could not get string from OsStr")),
    })
}

fn is_already_processed(filename: &String) -> bool {
    filename.len() == 64
        && filename
            .chars()
            .all(|c| c.is_ascii_digit() || (c.is_ascii_alphabetic() && c.is_ascii_lowercase()))
}

fn is_already_exists(filename: &String) -> bool {
    Path::new(filename).exists()
}

fn calculate_file_sha256(path: &Path) -> io::Result<String> {
    let file = fs::File::open(path)?;
    let mut reader = io::BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 1048576]; // 1 MB

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", base16ct::HexDisplay(&hasher.finalize())))
}
