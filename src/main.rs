use clap::{Parser, Subcommand};
use std::error::Error;

mod csv_to_fasta;
mod fasta_to_csv;
mod fastq_to_csv;
mod gzip_utils;
mod hash;

#[derive(Subcommand)]
enum SubCommands {
    Hash {
        #[clap(short = 'i', long = "input", value_name = "INPUT")]
        input: String,
        #[clap(short = 'o', long = "output", value_name = "OUTPUT")]
        output: String,
        #[clap(short = 'c', long = "csv", value_name = "CSV", required = false)]
        csv: Option<String>,
        #[clap(long = "gzip")]
        gzip: bool,
    },
    CsvToFasta {
        #[clap(short = 'i', long = "input", value_name = "INPUT")]
        input: String,
        #[clap(short = 'q', long = "header", value_name = "HEADER")]
        header: String,
        #[clap(short = 's', long = "sequence", value_name = "SEQUENCE")]
        sequence: String,
        #[clap(short = 'o', long = "output", value_name = "OUTPUT")]
        output: String,
    },
    FastaToCsv {
        #[clap(short = 'i', long = "input", value_name = "INPUT")]
        input: String,
        #[clap(short = 'o', long = "output", value_name = "OUTPUT")]
        output: String,
    },
    FastqToCsv {
        #[clap(short = 'i', long = "input", value_name = "INPUT")]
        input: String,
        #[clap(short = 'o', long = "output", value_name = "OUTPUT")]
        output: String,
        #[clap(long = "quality")]
        quality: bool,
    },
}

#[derive(clap::Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: SubCommands,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    match args.command {
        SubCommands::Hash {
            input,
            output,
            csv,
            gzip,
        } => {
            if let Err(e) = hash::run(input, output, csv, gzip) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        SubCommands::CsvToFasta {
            input,
            header,
            sequence,
            output,
        } => {
            csv_to_fasta::run(input, header, sequence, output)?;
        }
        SubCommands::FastaToCsv { input, output } => {
            fasta_to_csv::run(input, output)?;
        }
        SubCommands::FastqToCsv {
            input,
            output,
            quality,
        } => {
            fastq_to_csv::run(input, output, quality)?;
        }
    }

    Ok(())
}
