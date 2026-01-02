use clap::{Parser, ValueEnum};
use json2toon_rs::{decode, encode, DecoderOptions, Delimiter, EncoderOptions};
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(
    name = "json2toon",
    version,
    about = "Convert between JSON and TOON formats",
    long_about = "A fast, bidirectional converter between JSON and TOON (Token-Oriented Object Notation) formats.\n\
                  Supports reading from files or stdin and writing to files or stdout."
)]
struct Cli {
    /// Input file (stdin if not specified)
    #[arg(short, long, value_name = "FILE")]
    input: Option<PathBuf>,

    /// Output file (stdout if not specified)
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Conversion mode: encode (json->toon) or decode (toon->json)
    #[arg(short, long, value_enum, default_value = "encode")]
    mode: Mode,

    /// Delimiter for TOON format (only for encode mode)
    #[arg(short, long, value_enum, default_value = "comma")]
    delimiter: DelimiterArg,

    /// Indentation spaces (default: 2)
    #[arg(long, default_value = "2")]
    indent: usize,

    /// Strict mode for decoder (enforces counts and indentation)
    #[arg(long, default_value = "true")]
    strict: bool,

    /// Pretty print JSON output (only for decode mode)
    #[arg(short, long)]
    pretty: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    /// Convert JSON to TOON
    Encode,
    /// Convert TOON to JSON
    Decode,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum DelimiterArg {
    /// Comma delimiter (,)
    Comma,
    /// Tab delimiter (\t)
    Tab,
    /// Pipe delimiter (|)
    Pipe,
}

impl From<DelimiterArg> for Delimiter {
    fn from(arg: DelimiterArg) -> Self {
        match arg {
            DelimiterArg::Comma => Delimiter::Comma,
            DelimiterArg::Tab => Delimiter::Tab,
            DelimiterArg::Pipe => Delimiter::Pipe,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    // Read input
    let input = match read_input(&cli.input) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading input: {}", e);
            process::exit(1);
        }
    };

    // Process based on mode
    let output = match cli.mode {
        Mode::Encode => {
            // Parse JSON
            let json_value: serde_json::Value = match serde_json::from_str(&input) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Error parsing JSON: {}", e);
                    process::exit(1);
                }
            };

            // Encode to TOON
            let options = EncoderOptions {
                indent: cli.indent,
                delimiter: cli.delimiter.into(),
            };
            encode(&json_value, &options)
        }
        Mode::Decode => {
            // Decode from TOON
            let options = DecoderOptions {
                indent: cli.indent,
                strict: cli.strict,
            };
            let json_value = match decode(&input, &options) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Error decoding TOON: {}", e);
                    process::exit(1);
                }
            };

            // Convert to JSON string
            if cli.pretty {
                match serde_json::to_string_pretty(&json_value) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error serializing JSON: {}", e);
                        process::exit(1);
                    }
                }
            } else {
                match serde_json::to_string(&json_value) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error serializing JSON: {}", e);
                        process::exit(1);
                    }
                }
            }
        }
    };

    // Write output
    if let Err(e) = write_output(&cli.output, &output) {
        eprintln!("Error writing output: {}", e);
        process::exit(1);
    }
}

fn read_input(path: &Option<PathBuf>) -> io::Result<String> {
    match path {
        Some(file_path) => fs::read_to_string(file_path),
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
    }
}

fn write_output(path: &Option<PathBuf>, content: &str) -> io::Result<()> {
    match path {
        Some(file_path) => fs::write(file_path, content),
        None => {
            io::stdout().write_all(content.as_bytes())?;
            io::stdout().write_all(b"\n")?;
            io::stdout().flush()
        }
    }
}
