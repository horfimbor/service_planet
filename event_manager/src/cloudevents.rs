use super::Event;
use serde_json;
use uuid::Uuid;
use chrono::prelude::{Utc, DateTime};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CloudEvent {
    #[serde(rename = "specversion")]
    pub cloud_events_version: String,
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(rename = "typeversion")]
    pub event_type_version: String,
    pub source: String, // URI
    #[serde(rename = "id")]
    pub event_id: String,
    #[serde(rename = "time")]
    pub event_time: DateTime<Utc>,
    #[serde(rename = "datacontenttype")]
    pub content_type: String,
    pub subject: Option<String>,
    pub data: serde_json::Value,
}

impl<E> From<E> for CloudEvent
    where
        E: Event,
{
    fn from(source: E) -> Self {
        let raw_data = serde_json::to_string(&source).unwrap();

        let subject = match source.subject() {
            None => { None}
            Some(src) => {
                Some(src.to_string())
            }
        };

        CloudEvent {
            cloud_events_version: "1.0".to_owned(),
            event_type: source.event_type().to_owned(),
            event_type_version: source.event_type_version().to_owned(),
            source: source.event_source().to_owned(),
            event_id: Uuid::new_v4().to_hyphenated().to_string(),
            event_time: Utc::now(),
            content_type: "application/json".to_owned(),
            subject,
            data: serde_json::from_str(&raw_data).unwrap(),
        }
    }
}