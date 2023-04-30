use chrono::NaiveDate;
use serde::{de, Deserialize};

use super::Event;

pub struct ParseError<T> {
    expected: String,
    found: T
}

impl<T> ParseError<T> {
    pub fn new(expected_msg: &str, found: T) -> Self {
        Self { expected: expected_msg.to_owned(), found }
    }
}

impl<T: std::fmt::Display> std::fmt::Display for ParseError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "expected '{}' found '{}'", self.expected, self.found)
    }
}

#[derive(Debug, Deserialize)]
pub struct EventData {
    #[serde(rename = "NOME", deserialize_with = "non_empty_string")]
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

fn non_empty_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: de::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value.is_empty() {
        Err(de::Error::invalid_value(
            de::Unexpected::Str(&value),
            &"No empty string",
        ))
    } else {
        Ok(value)
    }
}

#[derive(Debug)]
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

fn parse_evt_desc<'de, D>(deserializer: D) -> Result<EventDesc, D::Error>
where
    D: de::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    EventDesc::try_from(value).map_err(de::Error::custom)
}

#[derive(Debug)]
pub enum EventDate {
    Day(NaiveDate),
    Period { start: NaiveDate, end: NaiveDate },
}

impl ToString for EventDate {
    fn to_string(&self) -> String {
        match self {
            Self::Day(day) => format!("dia {}", day.format("%d/%m/%Y")),
            Self::Period { start, end } => format!(
                "período de {} a {}",
                start.format("%d/%m/%Y"),
                end.format("%d/%m/%Y")
            ),
        }
    }
}

impl TryFrom<&str> for EventDate {
    type Error = chrono::ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.split_once('-') {
            Some((start, end)) => {
                let start = NaiveDate::parse_from_str(start.trim(), "%d/%m/%Y")?;
                let end = NaiveDate::parse_from_str(end.trim(), "%d/%m/%Y")?;
                Ok(Self::Period { start, end })
            }
            None => {
                let day = NaiveDate::parse_from_str(value.trim(), "%d/%m/%Y")?;
                Ok(Self::Day(day))
            }
        }
    }
}

fn parse_evt_date<'de, D>(deserializer: D) -> Result<EventDate, D::Error>
where
    D: de::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    EventDate::try_from(value.as_str()).map_err(de::Error::custom)
}

#[derive(Debug, Deserialize)]
pub struct Attendee {
    #[serde(rename = "NOME", deserialize_with = "parse_att_name")]
    pub name: String,
    #[serde(rename = "CPF", deserialize_with = "parse_cpf")]
    pub cpf: Cpf,
    #[serde(rename = "CH", deserialize_with = "parse_workload")]
    pub workload: u32,
}

fn parse_att_name<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: de::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value.is_empty() {
        Err(de::Error::invalid_value(
            de::Unexpected::Str(&value),
            &"No empty name",
        ))
    } else {
        // capitalize name
        let name = value
            .split_whitespace()
            .map(|word| word[0..1].to_uppercase() + &word[1..].to_lowercase())
            .collect::<Vec<_>>()
            .join(" ");
        Ok(name)
    }
}

fn parse_workload<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: de::Deserializer<'de>,
{
    let value = f64::deserialize(deserializer)?;
    if value < 0.0 {
        Err(de::Error::invalid_value(
            de::Unexpected::Float(value),
            &"Workload greater or equal than 0.0",
        ))
    } else {
        Ok(value.ceil() as u32)
    }
}

#[derive(Debug)]
pub struct Cpf {
    id: String,
}

impl Cpf {
    pub fn new(value: String) -> Option<Self> {
        for (i, part) in value.split('.').enumerate() {
            if i == 2 {
                let (left, right) = match part.split_once('-') {
                    Some(v) => v,
                    None => return None,
                };

                if left.len() != 3 || left.parse::<u64>().is_err() {
                    return None;
                }

                if right.len() != 2 || right.parse::<u16>().is_err() {
                    return None;
                }
            } else if part.len() != 3 && part.parse::<u16>().is_err() {
                return None;
            }
        }
        Some(Self { id: value })
    }

    pub fn as_str(&self) -> &str {
        &self.id
    }
}

fn parse_cpf<'de, D>(deserializer: D) -> Result<Cpf, D::Error>
where
    D: de::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    let err = de::Error::invalid_value(
        de::Unexpected::Str(&value),
        &"Valid CPF of the form 000.000.000-00",
    );
    Cpf::new(value).ok_or(err)
}
