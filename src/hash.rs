use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use hex::encode;
use serde_json::json;
use sha2::{Digest, Sha256};

pub fn run(input: String, output: String, output_json: Option<String>) {
    let reader = open_input_file(&input);
    let writer = open_output_file(&output);

    let records = read_fasta_records(reader);
    let (processed_records, header_mapping) = process_records(records);

    write_processed_records(writer, processed_records);

    if let Some(json_path) = output_json {
        write_header_mapping(&json_path, header_mapping);
    }
}

fn open_input_file(path: &str) -> BufReader<File> {
    let file = File::open(path).expect("Failed to open input file");
    BufReader::new(file)
}

fn open_output_file(path: &str) -> BufWriter<File> {
    let file = File::create(path).expect("Failed to create output file");
    BufWriter::new(file)
}

fn read_fasta_records(reader: BufReader<File>) -> Vec<(String, String)> {
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

fn process_records(records: Vec<(String, String)>) -> (Vec<String>, HashMap<String, String>) {
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

fn write_processed_records(mut writer: BufWriter<File>, records: Vec<String>) {
    for record in records {
        writer
            .write_all(record.as_bytes())
            .expect("Failed to write to output file");
    }
}

fn write_header_mapping(path: &str, header_mapping: HashMap<String, String>) {
    let json = json!(header_mapping);
    let mut file = File::create(path).expect("Failed to create JSON output file");
    file.write_all(json.to_string().as_bytes())
        .expect("Failed to write JSON to output file");
}

fn hash_header(header: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(header.as_bytes());
    let result = hasher.finalize();
    encode(result)[..12].to_string()
}
