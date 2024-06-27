use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use csv::WriterBuilder;

pub fn run(input: String, output: String) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(&input)?;
    let reader = BufReader::new(input_file);

    let mut wtr = WriterBuilder::new().has_headers(true).from_path(&output)?;

    wtr.write_record(&["header", "sequence"])?;

    let mut header = String::new();
    let mut sequence = String::new();

    for line in reader.lines() {
        let line = line?;
        if line.starts_with('>') {
            if !header.is_empty() {
                wtr.write_record(&[&header, &sequence])?;
            }
            header = line[1..].to_string();
            sequence.clear();
        } else {
            sequence.push_str(&line);
        }
    }

    if !header.is_empty() {
        wtr.write_record(&[&header, &sequence])?;
    }

    wtr.flush()?;
    Ok(())
}
