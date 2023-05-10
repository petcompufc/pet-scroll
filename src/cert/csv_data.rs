use serde::{de, Deserialize};
use time::{format_description::FormatItem, macros::format_description, Date};

use super::Event;

const DATE_FMT: &[FormatItem<'_>] = format_description!("[day]/[month]/[year]");

macro_rules! deserialize_fn {
    ($f:ident(): $fn:expr, $from:ty => $b:ty) => {
        fn $f<'de, D>(deserializer: D) -> Result<$b, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            let val = <$from>::deserialize(deserializer)?;
            $fn(val).map_err(de::Error::custom)
        }
    };
}

#[derive(Debug, PartialEq)]
pub struct ParseError<T> {
    expected: &'static str,
    found: T,
}

impl<T> ParseError<T> {
    pub fn new(expected_msg: &'static str, found: T) -> Self {
        Self {
            expected: expected_msg,
            found,
        }
    }
}

impl<T: std::fmt::Display> std::fmt::Display for ParseError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "expected '{}' found '{}'", self.expected, self.found)
    }
}

impl<T: std::fmt::Display + std::fmt::Debug> std::error::Error for ParseError<T> {}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct EventData {
    #[serde(rename = "NOME", deserialize_with = "parse_evt_name")]
    pub name: String,
    #[serde(rename = "DATA", deserialize_with = "parse_evt_date")]
    pub date: EventDate,
    #[serde(rename = "TEXTO", deserialize_with = "parse_evt_desc")]
    pub desc: EventDesc,
}

impl EventData {
    pub fn into_event(self, attendees: Vec<Attendee>) -> Event {
        Event {
            data: self,
            atts: attendees,
        }
    }
}

fn validate_evt_name(name: String) -> Result<String, ParseError<String>> {
    if name.is_empty() {
        let err = ParseError::new("Non-empty event name", name);
        return Err(err);
    }
    Ok(name)
}
deserialize_fn!(parse_evt_name(): validate_evt_name, String => String);

#[derive(Debug, Clone, PartialEq)]
pub enum EventDesc {
    Id(u32),
    Text(String),
}

impl TryFrom<String> for EventDesc {
    type Error = ParseError<String>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            let err = ParseError::new("Non-empty description Text or Id", value);
            return Err(err);
        }
        if let Ok(id) = value.parse::<u32>() {
            return Ok(EventDesc::Id(id));
        }
        Ok(EventDesc::Text(value))
    }
}
deserialize_fn!(parse_evt_desc(): EventDesc::try_from, String => EventDesc);

#[derive(Debug, Clone, PartialEq)]
pub enum EventDate {
    Day(Date),
    Period { start: Date, end: Date },
}

impl ToString for EventDate {
    fn to_string(&self) -> String {
        match self {
            Self::Day(date) => format!("dia {}", date.format(DATE_FMT).unwrap()),
            Self::Period { start, end } => format!(
                "período de {} a {}",
                start.format(DATE_FMT).unwrap(),
                end.format(DATE_FMT).unwrap()
            ),
        }
    }
}

impl TryFrom<String> for EventDate {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.split_once('-') {
            Some((start, end)) => {
                let start = Date::parse(start.trim(), DATE_FMT)?;
                let end = Date::parse(end.trim(), DATE_FMT)?;
                if start >= end {
                    Err(ParseError::new("Start of the event happen before the end", value).into())
                } else {
                    Ok(Self::Period { start, end })
                }
            }
            None => {
                let day = Date::parse(value.trim(), DATE_FMT)?;
                Ok(Self::Day(day))
            }
        }
    }
}
deserialize_fn!(parse_evt_date(): EventDate::try_from, String => EventDate);

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Attendee {
    #[serde(rename = "NOME", deserialize_with = "parse_att_name")]
    pub name: String,
    #[serde(rename = "CPF", deserialize_with = "parse_cpf")]
    pub cpf: Cpf,
    #[serde(rename = "CH", deserialize_with = "parse_workload")]
    pub workload: u32,
}

fn validate_att_name(name: String) -> Result<String, ParseError<String>> {
    if name.is_empty() {
        let err = ParseError::new("Non-empty attendee name", name);
        Err(err)
    } else {
        // capitalize name
        let name = name
            .split_whitespace()
            .map(|word| word[0..1].to_uppercase() + &word[1..].to_lowercase())
            .collect::<Vec<_>>()
            .join(" ");
        Ok(name)
    }
}
deserialize_fn!(parse_att_name(): validate_att_name, String => String);

fn validate_workload(workload: f64) -> Result<u32, ParseError<f64>> {
    if workload > 0.0 {
        Ok(workload.ceil() as u32)
    } else {
        Err(ParseError::new("Workload greater than 0.0", workload))
    }
}
deserialize_fn!(parse_workload(): validate_workload, f64 => u32);

#[derive(Debug, Clone, PartialEq)]
pub struct Cpf {
    id: String,
}

impl Cpf {
    pub fn new(value: String) -> Result<Self, ParseError<String>> {
        let value = value.trim();
        let err = Err(ParseError::new(
            "Valid CPF of the form 000.000.000-00",
            value.to_owned(),
        ));

        for (i, part) in value.split('.').enumerate() {
            if i == 2 {
                let (left, right) = match part.split_once('-') {
                    Some(v) => v,
                    None => return err,
                };

                if left.len() != 3 || left.parse::<u64>().is_err() {
                    return err;
                }

                if right.len() != 2 || right.parse::<u16>().is_err() {
                    return err;
                }
            } else if part.len() != 3 && part.parse::<u16>().is_err() {
                return err;
            }
        }
        Ok(Self {
            id: value.to_owned(),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.id
    }
}
deserialize_fn!(parse_cpf(): Cpf::new, String => Cpf);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_evt_name() {
        assert!(validate_evt_name("".to_owned()).is_err());
        assert_eq!(
            Ok("event name".to_owned()),
            validate_evt_name("event name".to_owned())
        );
    }

    #[test]
    fn check_evt_desc() {
        assert!(EventDesc::try_from("".to_owned()).is_err());

        let desc = EventDesc::try_from("20".to_owned()).expect("Should be a valid description Id");
        assert_eq!(EventDesc::Id(20), desc);

        let desc = EventDesc::try_from("Some text".to_owned())
            .expect("Should be a valid description Text");
        assert_eq!(EventDesc::Text("Some text".to_owned()), desc);
    }

    #[test]
    fn check_evt_date() {
        assert!(EventDate::try_from("01/01/2023-42/54/3050".to_owned()).is_err());
        assert!(EventDate::try_from("02/01/2023 - 01/01/2023".to_owned()).is_err());
        assert!(EventDate::try_from("02/01/2023 - 02/01/2023".to_owned()).is_err());

        let date =
            EventDate::try_from(" 01/01/2023  ".to_owned()).expect("Should be a valid event Day");
        assert_eq!("dia 01/01/2023", date.to_string());

        let date = EventDate::try_from(" 01/01/2023 - 04/04/2023 ".to_owned())
            .expect("Should be a valid event Period");
        assert_eq!(
            "período de 01/01/2023 a 04/04/2023".to_owned(),
            date.to_string()
        );
    }

    #[test]
    fn check_att_name() {
        assert!(validate_att_name("".to_owned()).is_err());
        assert_eq!(
            Ok("John Will".to_owned()),
            validate_att_name(" john  will ".to_owned())
        );
    }

    #[test]
    fn check_att_workload() {
        assert!(validate_workload(-1.0).is_err());
        assert!(validate_workload(0.0).is_err());
        assert_eq!(Ok(2), validate_workload(1.3));
        assert_eq!(Ok(5), validate_workload(5.0));
    }

    #[test]
    fn check_att_cpf() {
        assert!(Cpf::new("08911684350".to_owned()).is_err());
        assert!(Cpf::new("089.116.843.50".to_owned()).is_err());

        let cpf1 = Cpf::new("089.116.843-50".to_owned()).expect("it should be a valid CPF");
        assert_eq!("089.116.843-50", cpf1.as_str());

        let cpf2 = Cpf::new("   089.116.843-50 ".to_owned()).expect("it should be a valid CPF");
        assert_eq!(cpf1, cpf2);
    }
}
