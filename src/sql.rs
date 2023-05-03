pub trait ToSQL {
    fn to_sql(&self) -> QueryPool;
}

#[derive(Debug, Default, Clone, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_pool() {
        let queries = vec!["QUERY 1".to_owned(), "QUERY 2".to_owned()];
        let pool = QueryPool::new().queries(queries.clone());
        assert_eq!(queries, pool.into_iter().collect::<Vec<_>>());
    }

    #[test]
    fn pool_add() {
        let mut pool = QueryPool::new();
        pool.add("QUERY 1".to_owned());
        pool.add("QUERY 2".to_owned());
        assert_eq!(
            vec!["QUERY 1", "QUERY 2"],
            pool.into_iter().collect::<Vec<_>>()
        );
    }

    #[test]
    fn pool_add_many() {
        let queries = vec!["QUERY 1".to_owned(), "QUERY 2".to_owned()];
        let mut pool = QueryPool::new();
        pool.add_many(queries.clone());
        assert_eq!(queries, pool.into_iter().collect::<Vec<_>>());
    }

    #[test]
    fn pool_iter() {
        let queries = vec!["QUERY 1".to_owned(), "QUERY 2".to_owned()];
        let mut pool = QueryPool::new();
        pool.add_many(queries.clone());
        let result: Vec<_> = queries.clone().into_iter().collect();
        assert_eq!(queries, result);
    }

    #[test]
    fn pool_to_req() {
        let queries = vec!["QUERY 1".to_owned(), "QUERY 2".to_owned()];
        let pool = QueryPool::new().queries(queries);
        let req = pool.clone().into_req("database");
        assert_eq!(pool, req.queries);
    }

    #[test]
    fn pool_display() {
        let pool = QueryPool::new().queries(["QUERY 1".to_owned(), "QUERY 2".to_owned()]);
        let str = "QUERY 1;\nQUERY 2";
        assert_eq!(str, pool.to_string());
    }

    #[test]
    fn create_req() {
        let pool = QueryPool::new().queries(["QUERY 1".to_owned(), "QUERY 2".to_owned()]);
        let req = Request::new("database").queries(pool.clone());
        assert_eq!(pool, req.queries);
    }

    #[test]
    fn req_display() {
        let pool = QueryPool::new().queries(["QUERY 1".to_owned(), "QUERY 2".to_owned()]);
        let req = Request::new("database").queries(pool);
        let str = "USE database;\nQUERY 1;\nQUERY 2";
        assert_eq!(str, req.to_string());
    }
}
