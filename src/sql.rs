pub trait ToSQL {
    fn to_sql(&self) -> QueryPool;
}

#[derive(Debug, Default)]
pub struct QueryPool {
    pool: Vec<String>,
}

impl QueryPool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn query(mut self, query: String) -> Self {
        self.pool.push(query);
        self
    }

    pub fn queries<Q>(mut self, queries: Q) -> Self
    where
        Q: IntoIterator<Item = String>,
    {
        let pool = queries.into_iter().collect();
        self.pool = pool;
        self
    }

    pub fn add(&mut self, query: String) {
        self.pool.push(query);
    }

    pub fn add_many<Q>(&mut self, queries: Q)
    where
        Q: IntoIterator<Item = String>,
    {
        self.pool.extend(queries.into_iter());
    }

    pub fn into_req(self, db: &str) -> Request {
        Request::new(db).queries(self)
    }
}

impl IntoIterator for QueryPool {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;
    fn into_iter(self) -> Self::IntoIter {
        self.pool.into_iter()
    }
}

impl std::fmt::Display for QueryPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pool.join(";\n"))
    }
}

#[derive(Debug)]
pub struct Request {
    db: String,
    queries: QueryPool,
}

impl Request {
    pub fn new(db: &str) -> Self {
        Self {
            db: db.to_owned(),
            queries: QueryPool::new(),
        }
    }

    pub fn queries(mut self, queries: QueryPool) -> Self {
        self.queries = queries;
        self
    }
}

impl std::fmt::Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "USE {};\n{}", self.db, self.queries)
    }
}
