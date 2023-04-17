use clap::Parser;
use pet_cert::{
    event::{Attendee, EventData},
    sql::ToSQL,
};
use std::{io::Write, path::PathBuf};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Event Data CSV file.
    #[arg(short, long, value_parser = existing_file)]
    event: PathBuf,
    /// Attendees Info CSV file.
    #[arg(short, long = "atts", value_parser = existing_file)]
    attendees: PathBuf,
    /// An already uploaded event certificate image.
    #[arg(short, long, group = "image")]
    cert_img: Option<PathBuf>,
    /// Uploads the given event certificate image to the SFTP server.
    ///
    /// Upload the image to the SFTP server requires that SFTP_ADDRESS,
    /// SFTP_USER and SFTP_PWD environment variables are defined.
    /// It is recommended to use a .env file to store those credentials.
    #[arg(short, long, value_parser = existing_file, group = "image")]
    upload_img: Option<PathBuf>,
    /// SQL output file.
    #[arg(short, long, requires = "image")]
    output: PathBuf,
}

fn existing_file(s: &str) -> Result<PathBuf, String> {
    let path = std::path::Path::new(s);
    if !path.is_file() {
        return Err("is not a valid file".to_owned());
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
    let img_name = match (&args.cert_img, &args.upload_img) {
        (Some(img), None) | (None, Some(img)) => {
            let fname = img.file_name().unwrap().to_str().unwrap();
            fname.to_lowercase().replace(' ', "_")
        }
        _ => unreachable!("Both args should not be provided at the same time"),
    };

    print!("Reading event file...");
    std::io::stdout().flush().unwrap();
    let evt_file = std::fs::File::open(args.event).unwrap();
    let buffer = std::io::BufReader::new(evt_file);
    let evt = event_data(buffer);
    println!(" Done!");

    print!("Reading attendees file...");
    std::io::stdout().flush().unwrap();
    let atts_file = std::fs::File::open(args.attendees).unwrap();
    let buffer = std::io::BufReader::new(atts_file);
    let atts = attendees(buffer);
    println!(" Done!");

    let cert = evt.into_event(atts).into_cert(format!("img/{img_name}"));
    let queries = cert.to_sql("petcomp").into_queries();

    println!("Saving SQL queries at {}", args.output.display());
    std::fs::File::create(args.output)
        .expect("failed to create output file")
        .write_all(queries.as_bytes())
        .expect("failed to write into output file");

    if let Some(img) = args.upload_img {
        print!("Uploading event image...");
        std::io::stdout().flush().unwrap();

        // Verify environment variables.
        let _ = dotenvy::dotenv();
        let addr =
            std::env::var("SFTP_ADDRESS").expect("SFTP_ADDRESS environment variable not found");
        let user = std::env::var("SFTP_USER").expect("SFTP_USER environment variable not found");
        let pwd = std::env::var("SFTP_PWD").expect("SFTP_PWD environment variable not found");

        // Upload event image to the SFTP server
        let conn = pet_cert::sftp::connect(addr, &user, &pwd).unwrap();
        let remote_path = format!("./certificados/img/{img_name}");
        pet_cert::sftp::upload(&conn, img, remote_path).unwrap();
        println!(" Done!")
    }
}
