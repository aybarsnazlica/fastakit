use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

pub fn open_input_file(path: &str) -> Result<Box<dyn BufRead>, Box<dyn std::error::Error>> {
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
