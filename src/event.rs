use chrono::NaiveDate;
use serde::{de, Deserialize};

use crate::sql::{ToSQL, SQLReq};

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
}

impl ToSQL for Event {
    fn to_sql(&self) -> SQLReq {
        let mut req = SQLReq::new("petcomp");
        req.extend(self.data.to_sql()).unwrap();
        req.extend(self.atts.to_sql()).unwrap();

        // set event id
        req.add(format!(
            "SET @evid := (SELECT id FROM evento WHERE nome='{}' AND data='{}')",
            self.data.name,
            self.data.date.format("%d/%m/%Y")
        ));

        // set description id
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
                // set user id
                req.add(format!(
                    "SET @uid{i} := (SELECT id FROM usuario WHERE identificacao='{}')",
                    att.cpf.as_str()
                ));

                format!("(@uid{i}, @evid, @txtid, {})", att.workload)
            })
            .collect::<Vec<_>>()
            .join(",");
        req.add(format!(
            "INSERT INTO participacao (usuario, evento, ch, texto) VALUES {values}"
        ));
        req
    }
}

#[derive(Debug, Deserialize)]
pub struct EventData {
    #[serde(rename = "NOME", deserialize_with = "no_empty_string")]
    pub name: String,
    #[serde(rename = "DATA", deserialize_with = "validate_date")]
    pub date: NaiveDate,
    #[serde(rename = "TEXTO", deserialize_with = "parse_event_desc")]
    pub desc: EventDesc,
    #[serde(rename = "IMAGEM", deserialize_with = "no_empty_string")]
    pub img: String,
}

impl EventData {
    pub fn new(name: String, date: NaiveDate, desc: EventDesc, img_path: String) -> Self {
        Self {
            name,
            date,
            desc,
            img: img_path,
        }
    }

    pub fn as_event(self, attendees: Vec<Attendee>) -> Event {
        Event {
            data: self,
            atts: attendees,
        }
    }
}

impl ToSQL for EventData {
    fn to_sql(&self) -> SQLReq {
        let mut req = SQLReq::new("petcomp");
        req.add(format!(
            "INSERT INTO evento (nome, data, img) VALUES ('{}', '{}', '{}')",
            self.name,
            self.date.format("%d/%m/%Y"),
            self.img
        ));

        if let EventDesc::Text(txt) = &self.desc {
            req.add(format!("INSERT INTO texto VALUES ('{txt}')"));
        }
        req
    }
}

#[derive(Debug, Deserialize)]
pub enum EventDesc {
    Id(u32),
    Text(String),
}

#[derive(Debug, Deserialize)]
pub struct Attendee {
    #[serde(rename = "NOME", deserialize_with = "validate_att_name")]
    pub name: String,
    #[serde(rename = "CPF", deserialize_with = "validate_cpf")]
    pub cpf: Cpf,
    #[serde(rename = "CH", deserialize_with = "validate_workload")]
    // TODO: change to u32
    pub workload: f32,
}

impl ToSQL for Vec<Attendee> {
    fn to_sql(&self) -> SQLReq {
        let mut req = SQLReq::new("petcomp");
        let vals = self
            .iter()
            .map(|att| format!("('{}', '{}')", att.name, att.cpf.as_str()))
            .collect::<Vec<_>>()
            .join(",");
        req.add(format!("INSERT IGNORE INTO usuario (nome, identificacao) VALUES {vals}"));
        req
    }
}

#[derive(Debug, Deserialize)]
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

fn validate_workload<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: de::Deserializer<'de>,
{
    let value = f32::deserialize(deserializer)?;
    if value < 0.0 {
        Err(de::Error::invalid_value(
            de::Unexpected::Float(value as f64),
            &"Workload greater or equal than 0.0",
        ))
    } else {
        Ok(value)
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

fn validate_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: de::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&value, "%d/%m/%Y").map_err(de::Error::custom)
}
