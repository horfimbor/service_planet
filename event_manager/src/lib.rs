#[macro_use]
extern crate serde_derive;

use eventstore::EventData;

use serde::Serialize;
use serde_json::json;
use std::fmt;
use uuid::Uuid;

pub mod events;
pub mod aggregate;

pub use aggregate::AggregateState;
pub use events::GenericEvent;

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

    // TODO set async
    fn load_state(&self, subject: Uuid) -> Self::State;

    // TODO set async
    fn save_state(&self, state: Self::State) ->  Result<()>;

    // TODO set async
    fn load_events(&self, subject: Uuid, generation: u64) -> Vec<Self::Event>;

    // TODO set async
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
                self.save_events(events);
                Ok(())
            }
        }
    }
}