use chrono::NaiveDate;
use serde::{de, Deserialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct Attendee {
    #[serde(rename = "NOME", deserialize_with = "validate_att_name")]
    pub name: String,
    #[serde(deserialize_with = "validate_cpf")]
    pub cpf: Cpf,
    #[serde(rename = "CH", deserialize_with = "validate_workload")]
    pub workload: f32,
}

#[derive(Debug, Deserialize)]
pub struct Cpf {
    #[serde(rename = "CPF")]
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct Event {
    #[serde(rename = "NOME", deserialize_with = "no_empty_string")]
    name: String,
    #[serde(rename = "DATA", deserialize_with = "validate_date")]
    date: NaiveDate,
    #[serde(rename = "TEXTO", deserialize_with = "parse_event_desc")]
    desc: EventDesc,
    #[serde(rename = "IMAGEM", deserialize_with = "no_empty_string")]
    img: String,
}

impl Event {
    pub fn new(name: String, date: NaiveDate, desc: EventDesc, img_path: String) -> Self {
        Self {
            name,
            date,
            desc,
            img: img_path,
        }
    }
}

#[derive(Debug, Deserialize)]
pub enum EventDesc {
    Id(u32),
    Text(String),
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
            &"No empty name",
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
