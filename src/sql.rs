pub trait ToSQL {
    fn to_sql(&self, db: &str) -> SQLReq;
}

#[derive(Debug)]
pub struct SQLReq {
    db: String,
    queries: Vec<String>,
}

impl SQLReq {
    pub fn new(db: &str) -> Self {
        Self {
            db: db.to_owned(),
            queries: Vec::new(),
        }
    }

    pub fn add(&mut self, query: String) {
        self.queries.push(query);
    }

    pub fn extend(&mut self, req: SQLReq) -> Result<(), &'static str> {
        if self.db != req.db {
            return Err("req and self have different db's");
        }
        self.queries.extend(req.queries);
        Ok(())
    }

    pub fn into_queries(self) -> String {
        format!("USE {};\n{}", self.db, self.queries.join(";\n"))
    }
}
