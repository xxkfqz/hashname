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
#[command(
    name = "hashname",
    version,
    about = "Rename files to their hash",
    arg_required_else_help = true
)]
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
    /// Ignore file extensions when checking and renaming
    #[arg(short = 'i', long)]
    ignore_extension: bool,
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

    let checking_path = if opts.ignore_extension {
        path_file.file_name()
    } else {
        path_file.file_stem()
    };
    let file_name = get_str_from_osstr(&checking_path);
    if !opts.force_rehash && is_already_processed(&file_name.to_string()) {
        return Err(Box::from("Already processed"));
    }

    let result_hash = calculate_file_sha256(path_file)?;
    let mut result_filename = result_hash.clone();
    if !opts.ignore_extension {
        let file_ext = get_str_from_osstr(&path_file.extension());
        result_filename = match file_ext.len() {
            0 => result_hash,
            _ => format!("{}.{}", result_hash, file_ext),
        };
    }
    result_filename = result_filename.replace("\"", "\\\"");
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

fn get_str_from_osstr<'a>(osstr: &Option<&'a OsStr>) -> &'a str {
    match osstr {
        Some(n) => n.to_str().unwrap_or_else(|| ""),
        None => "",
    }
}

fn is_already_processed(filename: &String) -> bool {
    filename.len() == 64
        && filename
            .chars()
            .all(|c| c.is_ascii_digit() || (c.is_ascii_hexdigit() && c.is_ascii_lowercase()))
}

fn is_already_exists(filename: &String) -> bool {
    Path::new(filename).exists()
}

fn calculate_file_sha256(path: &Path) -> io::Result<String> {
    const BUFFER_SIZE: usize = 1048576; // 1 MB
    let file = fs::File::open(path)?;
    let mut reader = io::BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; BUFFER_SIZE];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", base16ct::HexDisplay(&hasher.finalize())))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hashed_name_detection() {
        let f = |s: &str| -> bool { is_already_processed(&String::from(s)) };

        assert_eq!(f("1"), false);
        assert_eq!(f("data1"), false);
        assert_eq!(
            f("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
            true
        );
        assert_eq!(
            f("0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF"),
            false
        );
        assert_eq!(
            f("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0"),
            false
        );
        assert_eq!(
            f("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcde"),
            false
        );
    }
}
