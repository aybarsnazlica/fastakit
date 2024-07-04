use std::error::Error;

use clap::{Parser, Subcommand};

mod csv_to_fasta;
mod fasta_to_csv;
mod hash;

#[derive(Subcommand)]
enum SubCommands {
    Hash {
        #[clap(short = 'i', long = "input", value_name = "INPUT")]
        input: String,
        #[clap(short = 'o', long = "output", value_name = "OUTPUT")]
        output: String,
        #[clap(short = 'c', long = "csv", value_name = "CSV", required = false)]
        json: Option<String>,
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
            json,
        } => {
            hash::run(input, output, json);
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
    }

    Ok(())
}
