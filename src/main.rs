use clap::Parser;
use events_cli::{
    event::{Attendee, EventData},
    sql::ToSQL,
};
use std::{io::Write, path::PathBuf};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Event Data CSV file
    #[arg(short, long, value_parser = existing_file)]
    event: PathBuf,
    /// Attendees Info CSV file
    #[arg(short, long = "atts", value_parser = existing_file)]
    attendees: PathBuf,
    /// Event certificate image
    ///
    /// Upload the image to the SFTP server requires that SFTP_ADDRESS,
    /// SFTP_USER and SFTP_PWD environment variables are defined.
    #[arg(short, long, value_parser = existing_file)]
    img: PathBuf,
    /// SQL queries output file
    #[arg(short, long)]
    output: PathBuf,
}

fn existing_file(s: &str) -> Result<PathBuf, String> {
    let path = std::path::Path::new(s);
    if !path.is_file() {
        return Err("is not a file".to_owned());
    }
    if !path.exists() {
        return Err("file does not exist".to_owned());
    }
    Ok(path.to_path_buf())
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
fn event_data<S>(src: S) -> EventData
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
    let _ = dotenvy::dotenv();

    // Verify environment variables.
    let addr = std::env::var("SFTP_ADDRESS").expect("SFTP_ADDRESS environment variable not found");
    let user = std::env::var("SFTP_USER").expect("SFTP_USER environment variable not found");
    let pwd = std::env::var("SFTP_PWD").expect("SFTP_PWD environment variable not found");

    let evt_file = std::fs::File::open(args.event).unwrap();
    let buffer = std::io::BufReader::new(evt_file);
    let evt = event_data(buffer);

    let atts_file = std::fs::File::open(args.attendees).unwrap();
    let buffer = std::io::BufReader::new(atts_file);
    let atts = attendees(buffer);

    let img_name = args.img.file_name().unwrap().to_str().unwrap();
    let evt = evt.as_event(atts, format!("img/{img_name}"));
    let queries = evt.to_sql().into_queries();

    std::fs::File::create(args.output)
        .expect("failed to create output file")
        .write_all(queries.as_bytes())
        .expect("failed to write into output file");

    // Upload event image to the SFTP server
    println!("Uploading event image...");
    let conn = events_cli::sftp::connect(addr, &user, &pwd).unwrap();
    let remote_path = format!("./certificados/img/{img_name}");
    events_cli::sftp::upload(&conn, args.img, remote_path).unwrap();
}
