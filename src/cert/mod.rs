use crate::sql::{SQLReq, ToSQL};

pub mod csv_data;
use csv_data::{Attendee, EventData, EventDesc};

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
