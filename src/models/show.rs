use serde_json::Value;
use sqlx::query::Query;

#[derive(Debug, Clone)]
pub struct Show {
    pub uri: String,
    pub name: String,
    pub publisher: String,
    pub description: Option<String>,
}

impl PartialEq for Show {
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri
    }
}

impl Eq for Show {}

impl Show {
    pub fn from_db_record(show_record: serde_json::Value) -> Result<Show, DataParseError> {
        Ok(Show {
            uri: show_record["uri"]
                .as_str()
                .ok_or(DataParseError::new("uri"))?
                .to_string(),
            name: show_record["name"]
                .as_str()
                .ok_or(DataParseError::new("name"))?
                .to_string(),
            publisher: show_record["publisher"]
                .as_str()
                .ok_or(DataParseError::new("publisher"))?
                .to_string(),
            description: show_record["description"].as_str().map(|s| s.to_string()),
        })
    }

    pub fn from_api_record(show_record: serde_json::Value) -> Result<Show, DataParseError> {
        // Same as from_db_record for this struct
        Show::from_db_record(show_record)
    }

    pub fn to_api_record(&self) -> serde_json::Value {
        serde_json::json!({
            "uri": self.uri,
            "name": self.name,
            "publisher": self.publisher,
            "description": self.description,
        })
    }

    pub fn generate_insert_query(&self) -> Query {
        let query_str = "INSERT INTO shows (uri, name, publisher, description) VALUES (?, ?, ?, ?)";
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.uri);
        query = query.bind(&self.name);
        query = query.bind(&self.publisher);
        query = query.bind(&self.description);
        query
    }

    pub fn generate_update_query(&self) -> Query {
        let query_str = "UPDATE shows SET name = ?, publisher = ?, description = ? WHERE uri = ?";
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.name);
        query = query.bind(&self.publisher);
        query = query.bind(&self.description);
        query = query.bind(&self.uri);
        query
    }

    pub fn generate_delete_query(&self) -> Query {
        let query_str = "DELETE FROM shows WHERE uri = ?";
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.uri);
        query
    }
}
