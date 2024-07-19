use crate::gzip_utils::open_input_file;
use csv::ReaderBuilder;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};

pub fn run(
    input: String,
    header: String,
    sequence: String,
    output: String,
) -> Result<(), Box<dyn Error>> {
    // Use open_input_file to handle gzipped input
    let reader = open_input_file(&input)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(reader);

    let headers = rdr.headers()?.clone();
    let header_idx = headers
        .iter()
        .position(|h| h == &header)
        .ok_or("Header column not found")?;
    let sequence_idx = headers
        .iter()
        .position(|h| h == &sequence)
        .ok_or("Sequence column not found")?;

    let mut wtr = File::create(&output)?;
    let mut writer = BufWriter::new(&mut wtr);

    for result in rdr.records() {
        let record = result?;
        let header_value = record.get(header_idx).ok_or("Missing header value")?;
        let sequence_value = record.get(sequence_idx).ok_or("Missing sequence value")?;
        writeln!(writer, ">{}\n{}", header_value, sequence_value)?;
    }

    Ok(())
}
