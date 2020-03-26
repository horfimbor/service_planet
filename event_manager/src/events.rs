use serde::{Deserialize, Serialize};

use serde_json;
use uuid::Uuid;
use chrono::prelude::{Utc, DateTime};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    id: Uuid,
    aggregate_id: Uuid,
    correlation_id: Uuid,
    causality_id: Uuid,
}

impl Metadata {
    pub fn new_for_command(aggregate_id: Uuid) -> Metadata {
        let id = Uuid::new_v4();

        Metadata {
            aggregate_id,
            id,
            correlation_id: id,
            causality_id: id,
        }
    }

    pub fn new_for_event(parent: Metadata) -> Metadata {
        Metadata {
            id: Uuid::new_v4(),
            aggregate_id: parent.aggregate_id,
            correlation_id: parent.correlation_id,
            causality_id: parent.id,
        }
    }
}


pub trait GenericEvent: Serialize {
    type Payload: Serialize;

    fn new(metadata: Metadata, payload: Self::Payload) -> Self;

    fn get_metadata(&self) -> Metadata;

    fn get_event_type(&self) -> String;

    fn is_command(&self) -> bool;

    fn get_payload(&self) -> Self::Payload;

    fn get_aggregate_id(&self) -> Uuid {
        self.get_metadata().aggregate_id
    }
}