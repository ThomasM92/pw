extern crate rpassword;

use argon2::Argon2;
use clap::Parser;
use clippers::Clipboard;
use rpassword::read_password;
use simple_crypt::{decrypt, encrypt};

use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    process::ExitCode,
};

/// PassWord based hashing and encryption tool
#[derive(Debug, Parser)]
#[command(long_about, arg_required_else_help = true)]
pub enum Cli {
    /// Hash a string with a salt and copy the output to clipboard
    Hash {
        /// Print the generated hash
        #[arg(short, long)]
        show: bool,

        /// Length of the output hash (default = 16, up to 64)
        length: Option<usize>,
    },

    /// Encrypt a file using the Advanced Encryption Strandard AES
    #[command(arg_required_else_help = true)]
    Encrypt {
        /// Copy password to clipboard
        #[arg(short, long)]
        copy: bool,

        /// Input file path
        input: PathBuf,

        /// Output file path (if not provided, input file is overwritten)
        output: Option<PathBuf>,
    },

    /// Decrypt a file using the Advanced Encryption Strandard AES
    #[command(arg_required_else_help = true)]
    Decrypt {
        /// Input file path
        input: PathBuf,

        /// Output file path (if not provided, input file is overwritten)
        output: Option<PathBuf>,
    },
}

// "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789£$&()*+[]@#^-_!?";
const CHARS: &'static str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789£$()*[]@^-_!?"; // no #&+

fn main() -> ExitCode {
    match app() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            println!("🐞 {}", err);
            ExitCode::FAILURE
        }
    }
}

fn app<'a>() -> Result<(), &'a str> {
    match Cli::parse() {
        Cli::Hash { show, length } => {
            print!("🔑 Password: ");
            std::io::stdout()
                .flush()
                .map_err(|_| "Could not flush STDOUT")?;
            let password = read_password().map_err(|_| "Could not read password")?;

            print!("🧂 Salt it: ");
            std::io::stdout()
                .flush()
                .map_err(|_| "Could not flush STDOUT")?;
            let salt = read_password().map_err(|_| "Could not read salt")?;

            let mut hash = [0u8; 64];
            Argon2::default()
                .hash_password_into(password.as_bytes(), salt.as_bytes(), &mut hash)
                .map_err(|_| "Hashing failed!")?;

            let l = length.unwrap_or(16).min(64);
            let c = CHARS.chars().count();
            let output: String = hash[0..l]
                .into_iter()
                .map(|b| CHARS.chars().nth(*b as usize % c).unwrap())
                .collect();

            match show {
                true => println!("✅ {}", &output),
                false => {
                    let mut clipboard = Clipboard::get();
                    clipboard
                        .write_text(&output)
                        .map_err(|_| "Could not copy password to clipboard")?;

                    println!("✅ Password copied to clipboard!");
                }
            };
        }
        Cli::Encrypt {
            copy,
            input,
            output,
        } => {
            let mut file = File::open(&input).map_err(|_| "Could not open input file")?;

            let mut buffer = String::new();
            file.read_to_string(&mut buffer)
                .map_err(|_| "Could not read input file")?;

            print!("🔑 Password: ");
            std::io::stdout()
                .flush()
                .map_err(|_| "Could not flush STDOUT")?;
            let password = read_password().map_err(|_| "Could not read password")?;

            if copy {
                let mut clipboard = Clipboard::get();
                clipboard
                    .write_text(&password)
                    .map_err(|_| "Could not copy password to clipboard")?;
            }

            let cypher = encrypt(buffer.as_bytes(), password.as_bytes())
                .map_err(|_| "Encryption failed!")?;

            match output {
                None => {
                    let mut wfile = OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .open(input)
                        .map_err(|_| "Could not open input file in write mode")?;
                    wfile
                        .write(&cypher)
                        .map_err(|_| "Could not write input file")?;
                }
                Some(path) => {
                    let mut output_file =
                        File::create_new(path).map_err(|_| "Could not create output file")?;
                    output_file
                        .write(&cypher)
                        .map_err(|_| "Could not write output file")?;
                }
            };

            println!("✅ File encrypted!");
        }
        Cli::Decrypt { input, output } => {
            let mut file = File::open(&input).map_err(|_| "Could not open input file")?;

            let mut buffer = Vec::<u8>::new();
            file.read_to_end(&mut buffer)
                .map_err(|_| "Could not read input file")?;

            print!("🔑 Password: ");
            std::io::stdout()
                .flush()
                .map_err(|_| "Could not flush STDOUT")?;
            let password = read_password().map_err(|_| "Could not read password")?;

            let plaintext =
                decrypt(&buffer, password.as_bytes()).map_err(|_| "Decryption failed!")?;

            match output {
                None => {
                    let mut wfile = OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .open(input)
                        .map_err(|_| "Could not open input file in write mode")?;
                    wfile
                        .write(&plaintext)
                        .map_err(|_| "Could not write input file")?;
                }
                Some(path) => {
                    let mut output_file =
                        File::create_new(path).map_err(|_| "Could not create output file")?;
                    output_file
                        .write(&plaintext)
                        .map_err(|_| "Could not write output file")?;
                }
            };

            println!("✅ File decrypted!");
        }
    };

    Ok(())
}
