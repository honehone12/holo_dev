use bson::serde_helpers::chrono_datetime_as_bson_datetime_optional;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Song {
    pub title: String,
    pub member: String,
    #[serde(with = "chrono_datetime_as_bson_datetime_optional")]
    pub published: Option<DateTime<Utc>>,
    pub links: Vec<String>,
    pub properties: Vec<String>
}
