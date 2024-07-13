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
) -> Result<(), Box<dyn std::error::Error>> {
    let reader = open_input_file(&input)?;
    let mut reader = BufReader::new(reader);
    let mut buffer = String::new();

    // Read the first line to determine the file type
    reader.read_line(&mut buffer)?;

    let file_type = if buffer.starts_with('>') {
        "FASTA"
    } else if buffer.starts_with('@') {
        "FASTQ"
    } else {
        return Err("Unknown file format".into());
    };

    // Rewind the reader to the beginning of the file
    let reader = BufReader::new(open_input_file(&input)?);

    let writer = open_output_file(&output)?;

    if file_type == "FASTA" {
        let records = read_fasta_records(reader);
        let (processed_records, header_mapping) = process_fasta_records(records);
        write_processed_records(writer, processed_records);

        if let Some(csv_path) = output_csv {
            write_header_mapping(&csv_path, header_mapping);
        }
    } else {
        let records = read_fastq_records(reader);
        let (processed_records, header_mapping) = process_fastq_records(records);
        write_processed_records(writer, processed_records);

        if let Some(csv_path) = output_csv {
            write_header_mapping(&csv_path, header_mapping);
        }
    }

    Ok(())
}

fn open_input_file(path: &str) -> Result<Box<dyn Read>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut buf = [0; 2];
    let mut reader = BufReader::new(&file);
    reader.read_exact(&mut buf)?;

    // Check if the file is gzipped
    if buf == [0x1f, 0x8b] {
        Ok(Box::new(GzDecoder::new(file)))
    } else {
        Ok(Box::new(File::open(path)?))
    }
}

fn open_output_file(path: &str) -> Result<BufWriter<GzEncoder<File>>, Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    let encoder = GzEncoder::new(file, Compression::default());
    Ok(BufWriter::new(encoder))
}

fn read_fasta_records(reader: BufReader<Box<dyn Read>>) -> Vec<(String, String)> {
    let mut records = Vec::new();
    let mut lines = reader.lines();
    let mut current_header = String::new();
    let mut current_sequence = String::new();

    while let Some(Ok(line)) = lines.next() {
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

    records
}

fn read_fastq_records(reader: BufReader<Box<dyn Read>>) -> Vec<(String, String, String)> {
    let mut records = Vec::new();
    let mut lines = reader.lines();
    while let (Some(Ok(header)), Some(Ok(sequence)), Some(Ok(plus)), Some(Ok(quality))) =
        (lines.next(), lines.next(), lines.next(), lines.next())
    {
        if header.starts_with('@') && plus.starts_with('+') {
            records.push((header, sequence, quality));
        } else {
            panic!("Malformed FASTQ file");
        }
    }
    records
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

fn write_processed_records(mut writer: BufWriter<GzEncoder<File>>, records: Vec<String>) {
    for record in records {
        writer
            .write_all(record.as_bytes())
            .expect("Failed to write to output file");
    }
}

fn write_header_mapping(path: &str, header_mapping: HashMap<String, String>) {
    let mut wtr = csv::Writer::from_path(path).expect("Failed to create CSV output file");

    // Write the header row
    wtr.write_record(&["header_original", "header_hashed"])
        .expect("Failed to write header row to CSV");

    // Write the header mappings
    for (original, hashed) in header_mapping {
        wtr.write_record(&[original, hashed])
            .expect("Failed to write record to CSV");
    }

    wtr.flush().expect("Failed to flush CSV writer");
}

fn hash_header(header: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(header.as_bytes());
    let result = hasher.finalize();
    encode(result)[..12].to_string()
}
