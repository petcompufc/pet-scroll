use events_cli::Attendee;

fn attendees() -> Vec<Attendee> {
    let mut rdr = csv::Reader::from_reader(std::io::stdin());
    rdr.deserialize()
        .map(|result| result.expect("Error while parsing STDIN"))
        .collect::<Vec<Attendee>>()
}

fn main() -> std::io::Result<()> {
    let atts = attendees();
    println!("{atts:#?}");
    Ok(())
}
