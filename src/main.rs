use clap::Parser;
use events_cli::{Attendee, Event};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Event Data CSV file
    #[arg(short, long, value_parser = is_valid_file)]
    event: PathBuf,
    /// Attendees Info CSV file
    #[arg(short, long = "atts", value_parser = is_valid_file)]
    attendees: PathBuf,
}

fn is_valid_file(s: &str) -> Result<PathBuf, String> {
    let path = std::path::Path::new(s);
    if !path.exists() {
        return Err("file does not exist".to_owned());
    }

    match path.extension() {
        Some(ext) if ext == "csv" => Ok(path.to_path_buf()),
        _ => Err("the file must be a CSV file".to_owned()),
    }
}

/// Read event attendees from the given `src`.
fn attendees<S>(src: S) -> Vec<Attendee>
where
    S: std::io::Read,
{
    let mut rdr = csv::Reader::from_reader(src);
    rdr.deserialize()
        .map(|result| result.expect("Error while parsing STDIN"))
        .collect::<Vec<Attendee>>()
}

/// Read event from the given `src`.
fn event<S>(src: S) -> Event
where
    S: std::io::Read,
{
    let mut rdr = csv::Reader::from_reader(src);
    rdr.deserialize()
        .next()
        .expect("No event info found")
        .expect("Error while parsing STDIN")
}

fn main() {
    let args = Args::parse();
    let atts_file = std::fs::File::open(args.attendees).unwrap();
    let buffer = std::io::BufReader::new(atts_file);
    let atts = attendees(buffer);

    let evt_file = std::fs::File::open(args.event).unwrap();
    let buffer = std::io::BufReader::new(evt_file);
    let evt = event(buffer);
    println!("{:?}\n", atts);
    println!("{:?}", evt);
}
