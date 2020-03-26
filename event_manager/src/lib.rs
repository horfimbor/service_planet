use std::fmt;
use uuid::Uuid;

pub mod events;

pub use events::{GenericEvent, Metadata};


#[cfg(feature = "eventstore-rs")]
use futures::executor::block_on;

#[cfg(feature = "eventstore-rs")]
use eventstore::Connection;

#[cfg(feature = "eventstore-rs")]
use tokio::stream::StreamExt;

#[cfg(feature = "eventstore-rs")]
use std::str;

#[cfg(feature = "eventstore-rs")]
use serde_json::json;

pub trait AggregateState: Clone + Default {
    fn generation(&self) -> u64;
}

#[derive(Debug)]
pub struct Error {
    pub kind: Kind,
}

/// Indicates the kind of event sourcing error that occurred.
#[derive(Debug)]
pub enum Kind {
    ApplicationFailure(String),
    CommandFailure(String),
    StoreFailure(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            Kind::ApplicationFailure(ref s) => fmt::Display::fmt(s, f),
            Kind::CommandFailure(ref s) => fmt::Display::fmt(s, f),
            Kind::StoreFailure(ref s) => fmt::Display::fmt(s, f),
        }
    }
}

/// A Result where failure is an event sourcing error
pub type Result<T> = std::result::Result<T, Error>;

pub trait Aggregate
{
    type Event: GenericEvent;
    type Command: GenericEvent;
    type State: AggregateState;

    #[cfg(feature = "eventstore-rs")]
    fn get_aggregate_prefix() -> &'static str;

    #[cfg(feature = "eventstore-rs")]
    fn get_connection(&self) -> &Connection;

    fn load_state(&self, subject: Uuid) -> Self::State;

    fn save_state(&self, state: Self::State) -> Result<()>;

    #[cfg(not(feature = "eventstore-rs"))]
    fn load_events(&self, subject: Uuid, generation: u64) -> Vec<Self::Event>;

    #[cfg(not(feature = "eventstore-rs"))]
    fn save_events(&self, events: Vec<Self::Event>) -> Result<()>;

    // apply event must be stateless
    fn apply_event(state: &Self::State, evt: &Self::Event) -> Result<Self::State>;

    // apply command must be stateless
    fn apply_command(state: &Self::State, cmd: &Self::Command) -> Result<Vec<Self::Event>>;

    fn apply_all(state: &Self::State, evts: Vec<Self::Event>) -> Result<Self::State> {
        Ok(evts.iter().fold(state.clone(), |acc_state, event| {
            Self::apply_event(&acc_state, event).unwrap()
        }))
    }

    fn get_up_to_date_state(&self, aggregate_id: Uuid) -> Result<Self::State> {
        let state = self.load_state(aggregate_id);

        let events = self.load_events(aggregate_id, state.generation());

        Self::apply_all(&state, events)
    }

    // this is real the entry point
    fn handle_command(&self, cmd: &Self::Command) -> Result<()> {
        let state = self.get_up_to_date_state(cmd.get_aggregate_id())?;

        match Self::apply_command(&state, cmd) {
            Err(err) => {
                Err(err)
            }
            Ok(events) => {
                self.save_events(events)
            }
        }
    }


    #[cfg(feature = "eventstore-rs")]
    fn load_events(&self, subject: Uuid, _generation: u64) -> Vec<Self::Event> {
        let mut vec = Vec::new();

        block_on(async {
            let stream_id = format!("{}{}", Self::get_aggregate_prefix(), subject);

            let mut stream = self.get_connection()
                .read_stream(stream_id.as_str())
                .start_from_beginning()
                .max_count(1)
                .iterate_over();

            while let Some(event) = stream.try_next().await.unwrap() {
                let recorder_event = event.get_original_event();

                let data_str = str::from_utf8(&recorder_event.data).unwrap().to_string();
                let meta_str = str::from_utf8(&recorder_event.metadata).unwrap().to_string();

                let data: <<Self as Aggregate>::Event as GenericEvent>::Payload = serde_json::from_str(data_str.as_str()).unwrap();
                let metadata: Metadata = serde_json::from_str(meta_str.as_str()).unwrap();

                let self_event = Self::Event::new(metadata, data);

                vec.push(self_event);
            }
        });
        vec
    }

    #[cfg(feature = "eventstore-rs")]
    fn save_events(&self, events: Vec<Self::Event>) -> Result<()>{
        block_on(async {
            for event in &events {
                let stream_id = format!("{}{}", Self::get_aggregate_prefix(), event.get_aggregate_id());

                let payload = json!( event.get_payload() );
                let metadata = json!(event.get_metadata());

                let data = eventstore::EventData::json(event.get_event_type(), payload).unwrap();

                let data = data.metadata_as_json(metadata);

                let result = self.get_connection()
                    .write_events(stream_id)
                    .push_event(data)
                    .execute()
                    .await.unwrap();
            }
            Ok(())
        })
    }
}