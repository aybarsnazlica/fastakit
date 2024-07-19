use crate::gzip_utils::open_input_file;
use csv::WriterBuilder;
use std::error::Error;
use std::io::BufRead;

pub fn run(input: String, output: String, write_quality: bool) -> Result<(), Box<dyn Error>> {
    let reader = open_input_file(&input)?;

    let mut wtr = WriterBuilder::new().has_headers(true).from_path(&output)?;

    if write_quality {
        wtr.write_record(&["header", "sequence", "quality"])?;
    } else {
        wtr.write_record(&["header", "sequence"])?;
    }

    let mut lines = reader.lines();
    while let Some(header) = lines.next() {
        let header = header?;
        let sequence = lines.next().ok_or("Missing sequence")??;
        let _plus = lines.next().ok_or("Missing + line")??;
        let quality = lines.next().ok_or("Missing quality scores")??;

        if write_quality {
            wtr.write_record(&[&header[1..], &sequence, &quality])?;
        } else {
            wtr.write_record(&[&header[1..], &sequence])?;
        }
    }

    wtr.flush()?;
    Ok(())
}
