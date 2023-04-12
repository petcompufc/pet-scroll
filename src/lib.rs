use serde::{de, Deserialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct Attendee {
    #[serde(rename = "NOME")]
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
