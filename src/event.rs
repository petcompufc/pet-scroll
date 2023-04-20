use chrono::NaiveDate;
use serde::{de, Deserialize};

use crate::sql::{SQLReq, ToSQL};

#[derive(Debug)]
pub struct Certificate {
    event: Event,
    img: String,
}

impl ToSQL for Certificate {
    fn to_sql(&self, db: &str) -> SQLReq {
        let mut req = SQLReq::new(db);
        req.add(format!(
            "INSERT INTO evento (nome, data, img) VALUES ('{}', '{}', '{}')",
            self.event.data.name,
            self.event.data.date.to_string(),
            self.img
        ));
        req.extend(self.event.to_sql(db)).unwrap();
        req
    }
}

#[derive(Debug)]
pub struct Event {
    data: EventData,
    atts: Vec<Attendee>,
}

impl Event {
    pub fn data(&self) -> &EventData {
        &self.data
    }

    pub fn attendees(&self) -> &[Attendee] {
        &self.atts
    }

    pub fn into_cert(self, img: String) -> Certificate {
        Certificate { event: self, img }
    }
}

impl ToSQL for Event {
    fn to_sql(&self, db: &str) -> SQLReq {
        let mut req = SQLReq::new(db);
        req.extend(self.atts.to_sql(db)).unwrap();

        if let EventDesc::Text(txt) = &self.data.desc {
            req.add(format!("INSERT INTO texto (texto) VALUES ('{txt}')"));
        }

        // get event id
        req.add(format!(
            "SET @evid := (SELECT id FROM evento WHERE nome='{}' AND data='{}')",
            self.data.name,
            self.data.date.to_string()
        ));

        // get description id
        let part = match &self.data.desc {
            EventDesc::Id(id) => format!("SET @txtid = {id}"),
            EventDesc::Text(txt) => {
                format!("SET @txtid := (SELECT id FROM texto WHERE texto='{txt}')")
            }
        };
        req.add(part);

        let values = self
            .atts
            .iter()
            .enumerate()
            .map(|(i, att)| {
                // get user id
                req.add(format!(
                    "SET @uid{i} := (SELECT id FROM usuario WHERE identificacao='{}')",
                    att.cpf.as_str()
                ));
                format!("(@uid{i}, @evid, @txtid, {})", att.workload)
            })
            .collect::<Vec<_>>()
            .join(",");
        req.add(format!(
            "INSERT INTO participacao (usuario, evento, texto, ch) VALUES {values}"
        ));
        req
    }
}

#[derive(Debug, Deserialize)]
pub struct EventData {
    #[serde(rename = "NOME", deserialize_with = "no_empty_string")]
    pub name: String,
    #[serde(rename = "DATA", deserialize_with = "evt_date_parser")]
    pub date: EventDate,
    #[serde(rename = "TEXTO", deserialize_with = "parse_event_desc")]
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

#[derive(Debug)]
pub enum EventDesc {
    Id(u32),
    Text(String),
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

#[derive(Debug, Deserialize)]
pub struct Attendee {
    #[serde(rename = "NOME", deserialize_with = "validate_att_name")]
    pub name: String,
    #[serde(rename = "CPF", deserialize_with = "validate_cpf")]
    pub cpf: Cpf,
    #[serde(rename = "CH", deserialize_with = "validate_workload")]
    pub workload: u32,
}

impl ToSQL for Vec<Attendee> {
    fn to_sql(&self, db: &str) -> SQLReq {
        let mut req = SQLReq::new(db);
        let vals = self
            .iter()
            .map(|att| format!("('{}', '{}')", att.name, att.cpf.as_str()))
            .collect::<Vec<_>>()
            .join(",");
        req.add(format!(
            "INSERT IGNORE INTO usuario (nome, identificacao) VALUES {vals}"
        ));
        req
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

fn validate_att_name<'de, D>(deserializer: D) -> Result<String, D::Error>
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

fn validate_cpf<'de, D>(deserializer: D) -> Result<Cpf, D::Error>
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

fn validate_workload<'de, D>(deserializer: D) -> Result<u32, D::Error>
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

fn parse_event_desc<'de, D>(deserializer: D) -> Result<EventDesc, D::Error>
where
    D: de::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if let Ok(id) = value.parse::<u32>() {
        return Ok(EventDesc::Id(id));
    }
    Ok(EventDesc::Text(value))
}

fn no_empty_string<'de, D>(deserializer: D) -> Result<String, D::Error>
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

fn evt_date_parser<'de, D>(deserializer: D) -> Result<EventDate, D::Error>
where
    D: de::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    EventDate::try_from(value.as_str()).map_err(de::Error::custom)
}
