use crate::sql::{QueryPool, ToSQL};

pub mod csv_data;
use csv_data::{Attendee, EventData, EventDesc};

#[derive(Debug)]
pub struct Certificate {
    event: Event,
    img: String,
}

impl ToSQL for Certificate {
    fn to_sql(&self) -> QueryPool {
        let mut pool = QueryPool::new();

        // insert event
        pool.add(format!(
            "INSERT IGNORE INTO evento (nome, data, img) VALUES ('{}', '{}', '{}')",
            self.event.data.name,
            self.event.data.date.to_string(),
            self.img
        ));

        // get event id
        pool.add(format!(
            "SET @evid := (SELECT id FROM evento WHERE nome='{}' AND data='{}' AND img='{}')",
            self.event.data.name,
            self.event.data.date.to_string(),
            self.img
        ));

        // add event queries
        pool.add_many(self.event.to_sql());
        pool
    }
}

#[derive(Debug, Clone, PartialEq)]
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
    fn to_sql(&self) -> QueryPool {
        let mut pool = QueryPool::new();
        pool.add_many(self.atts.to_sql());

        // insert event text
        if let EventDesc::Text(txt) = &self.data.desc {
            pool.add(format!("INSERT IGNORE INTO texto (texto) VALUES ('{txt}')"));
        }

        // get description id
        let part = match &self.data.desc {
            EventDesc::Id(id) => format!("SET @txtid = {id}"),
            EventDesc::Text(txt) => {
                format!("SET @txtid := (SELECT id FROM texto WHERE texto='{txt}')")
            }
        };
        pool.add(part);

        let values = self
            .atts
            .iter()
            .enumerate()
            .map(|(i, att)| {
                // get user id
                pool.add(format!(
                    "SET @uid{i} := (SELECT id FROM usuario WHERE identificacao='{}')",
                    att.cpf.as_str()
                ));
                format!("(@uid{i}, @evid, @txtid, {})", att.workload)
            })
            .collect::<Vec<_>>()
            .join(",");
        pool.add(format!(
            "INSERT INTO participacao (usuario, evento, texto, ch) VALUES {values}"
        ));
        pool
    }
}

impl ToSQL for Vec<Attendee> {
    fn to_sql(&self) -> QueryPool {
        let mut pool = QueryPool::new();
        let vals = self
            .iter()
            .map(|att| format!("('{}', '{}')", att.name, att.cpf.as_str()))
            .collect::<Vec<_>>()
            .join(",");
        pool.add(format!(
            "INSERT IGNORE INTO usuario (nome, identificacao) VALUES {vals}"
        ));
        pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cert::csv_data::{Cpf, EventData, EventDate, EventDesc};
    use time::macros::date;

    #[test]
    fn atts_to_sql() {
        let att_a = Attendee {
            name: "A".to_owned(),
            cpf: Cpf::new("207.062.844-29".to_owned()).expect("valid cpf"),
            workload: 1,
        };
        let att_b = Attendee {
            name: "B".to_owned(),
            cpf: Cpf::new("647.748.630-09".to_owned()).expect("valid cpf"),
            workload: 1,
        };
        let pool = vec![att_a, att_b].to_sql();

        let vals = "('A', '207.062.844-29'),('B', '647.748.630-09')";
        let result = format!("INSERT IGNORE INTO usuario (nome, identificacao) VALUES {vals};\n");

        assert_eq!(result, pool.to_string());
    }

    #[test]
    fn event_to_sql() {
        let data = EventData {
            name: "Event".to_owned(),
            desc: EventDesc::Text("Some description".to_owned()),
            date: EventDate::Day(date!(2023 - 05 - 04)),
        };
        let att_a = Attendee {
            name: "A".to_owned(),
            cpf: Cpf::new("754.751.875-33".to_owned()).expect("valid cpf"),
            workload: 1,
        };
        let att_b = Attendee {
            name: "B".to_owned(),
            cpf: Cpf::new("647.748.630-09".to_owned()).expect("valid cpf"),
            workload: 2,
        };
        let atts = vec![att_a, att_b];
        let atts_sql = atts.to_sql().to_string();

        let event = data.into_event(atts);

        let mut result = atts_sql;
        result.push_str(
            &[
                "INSERT IGNORE INTO texto (texto) VALUES ('Some description')",
                "SET @txtid := (SELECT id FROM texto WHERE texto='Some description')",
                "SET @uid0 := (SELECT id FROM usuario WHERE identificacao='754.751.875-33')",
                "SET @uid1 := (SELECT id FROM usuario WHERE identificacao='647.748.630-09')",
                "INSERT INTO participacao (usuario, evento, texto, ch) \
                VALUES (@uid0, @evid, @txtid, 1),(@uid1, @evid, @txtid, 2);\n",
            ]
            .join(";\n"),
        );

        assert_eq!(result, event.to_sql().to_string());
    }

    #[test]
    fn create_cert() {
        let data = EventData {
            name: "Event".to_owned(),
            desc: EventDesc::Text("Some description".to_owned()),
            date: EventDate::Day(date!(2023 - 5 - 4)),
        };
        let att_a = Attendee {
            name: "A".to_owned(),
            cpf: Cpf::new("754.751.875-33".to_owned()).expect("valid cpf"),
            workload: 1,
        };
        let att_b = Attendee {
            name: "B".to_owned(),
            cpf: Cpf::new("647.748.630-09".to_owned()).expect("valid cpf"),
            workload: 2,
        };
        let event = data.into_event(vec![att_a, att_b]);
        let cert = event.clone().into_cert("cert.png".to_owned());

        assert_eq!(cert.img, "cert.png".to_owned());
        assert_eq!(cert.event, event);
    }

    #[test]
    fn cert_to_sql() {
        let data = EventData {
            name: "Event".to_owned(),
            desc: EventDesc::Text("Some description".to_owned()),
            date: EventDate::Day(date!(2023 - 5 - 4)),
        };
        let att_a = Attendee {
            name: "A".to_owned(),
            cpf: Cpf::new("754.751.875-33".to_owned()).expect("valid cpf"),
            workload: 1,
        };
        let att_b = Attendee {
            name: "B".to_owned(),
            cpf: Cpf::new("647.748.630-09".to_owned()).expect("valid cpf"),
            workload: 2,
        };
        let event = data.into_event(vec![att_a, att_b]);
        let event_sql = event.to_sql().to_string();

        let cert = event.into_cert("cert.png".to_owned());

        let result = [
            "INSERT IGNORE INTO evento (nome, data, img) VALUES ('Event', 'dia 04/05/2023', 'cert.png')",
            "SET @evid := (SELECT id FROM evento WHERE nome='Event' AND data='dia 04/05/2023' AND img='cert.png')",
            &event_sql,
        ]
        .join(";\n");

        assert_eq!(result, cert.to_sql().to_string());
    }
}
