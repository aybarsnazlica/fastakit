use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};

use hex::encode;
use sha2::{Digest, Sha256};

pub fn run(
    input: String,
    output: String,
    output_csv: Option<String>,
    gzip: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let reader = open_input_file(&input)?;
    let mut buffer = String::new();

    // Read the first line to determine the file type
    let mut peekable_reader = reader.take(1);
    peekable_reader.read_to_string(&mut buffer)?;

    let file_type = if buffer.starts_with('>') {
        "FASTA"
    } else if buffer.starts_with('@') {
        "FASTQ"
    } else {
        return Err("Unknown file format".into());
    };

    // Rewind the reader to the beginning of the file
    let reader = open_input_file(&input)?;

    let writer = open_output_file(&output, gzip)?;

    if file_type == "FASTA" {
        let records = read_fasta_records(reader)?;
        let (processed_records, header_mapping) = process_fasta_records(records);
        write_processed_records(writer, processed_records)?;

        if let Some(csv_path) = output_csv {
            write_header_mapping(&csv_path, header_mapping)?;
        }
    } else {
        let records = read_fastq_records(reader)?;
        let (processed_records, header_mapping) = process_fastq_records(records);
        write_processed_records(writer, processed_records)?;

        if let Some(csv_path) = output_csv {
            write_header_mapping(&csv_path, header_mapping)?;
        }
    }

    Ok(())
}

fn open_input_file(path: &str) -> Result<Box<dyn BufRead>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut buf = [0; 2];
    let mut reader = BufReader::new(&file);
    reader.read_exact(&mut buf)?;

    // Check if the file is gzipped
    if buf == [0x1f, 0x8b] {
        let file = File::open(path)?;
        Ok(Box::new(BufReader::new(GzDecoder::new(file))))
    } else {
        let file = File::open(path)?;
        Ok(Box::new(BufReader::new(file)))
    }
}

fn open_output_file(path: &str, gzip: bool) -> Result<Box<dyn Write>, Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    if gzip {
        let encoder = GzEncoder::new(file, Compression::default());
        Ok(Box::new(BufWriter::new(encoder)))
    } else {
        Ok(Box::new(BufWriter::new(file)))
    }
}

fn read_fasta_records<R: BufRead>(
    reader: R,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let mut records = Vec::new();
    let mut lines = reader.lines();
    let mut current_header = String::new();
    let mut current_sequence = String::new();

    while let Some(line) = lines.next() {
        let line = line?;
        if line.starts_with('>') {
            if !current_header.is_empty() {
                records.push((current_header.clone(), current_sequence.clone()));
            }
            current_header = line;
            current_sequence.clear();
        } else {
            current_sequence.push_str(&line);
        }
    }

    if !current_header.is_empty() {
        records.push((current_header, current_sequence));
    }

    Ok(records)
}

fn read_fastq_records<R: BufRead>(
    reader: R,
) -> Result<Vec<(String, String, String)>, Box<dyn std::error::Error>> {
    let mut records = Vec::new();
    let mut lines = reader.lines();
    while let (Some(header), Some(sequence), Some(plus), Some(quality)) =
        (lines.next(), lines.next(), lines.next(), lines.next())
    {
        let header = header?;
        let sequence = sequence?;
        let plus = plus?;
        let quality = quality?;
        if header.starts_with('@') && plus.starts_with('+') {
            records.push((header, sequence, quality));
        } else {
            return Err("Malformed FASTQ file".into());
        }
    }
    Ok(records)
}

fn process_fasta_records(records: Vec<(String, String)>) -> (Vec<String>, HashMap<String, String>) {
    let mut header_mapping = HashMap::new();
    let processed_records: Vec<String> = records
        .iter()
        .map(|(header, sequence)| {
            let original_header = header[1..].to_string(); // Remove '>' from header
            let header_hash = hash_header(&original_header);
            header_mapping.insert(original_header.clone(), header_hash.clone());
            format!(">{}\n{}\n", header_hash, sequence)
        })
        .collect();
    (processed_records, header_mapping)
}

fn process_fastq_records(
    records: Vec<(String, String, String)>,
) -> (Vec<String>, HashMap<String, String>) {
    let mut header_mapping = HashMap::new();
    let processed_records: Vec<String> = records
        .iter()
        .map(|(header, sequence, quality)| {
            let original_header = header[1..].to_string(); // Remove '@' from header
            let header_hash = hash_header(&original_header);
            header_mapping.insert(original_header.clone(), header_hash.clone());
            format!("@{}\n{}\n+\n{}\n", header_hash, sequence, quality)
        })
        .collect();
    (processed_records, header_mapping)
}

fn write_processed_records<W: Write>(
    mut writer: W,
    records: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    for record in records {
        writer.write_all(record.as_bytes())?;
    }
    Ok(())
}

fn write_header_mapping(
    path: &str,
    header_mapping: HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_path(path)?;

    // Write the header row
    wtr.write_record(&["header_original", "header_hashed"])?;

    // Write the header mappings
    for (original, hashed) in header_mapping {
        wtr.write_record(&[original, hashed])?;
    }

    wtr.flush()?;
    Ok(())
}

fn hash_header(header: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(header.as_bytes());
    let result = hasher.finalize();
    encode(result)[..12].to_string()
}
