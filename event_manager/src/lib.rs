
#[macro_use]
extern crate serde_derive;

use serde::Serialize;
use std::fmt;
use uuid::Uuid;

pub mod cloudevents;

pub trait Command {
    fn event_type_version(&self) -> &str;
    fn event_type(&self) -> &str;
    fn event_source(&self) -> &str;
    fn is_valid(&self) -> bool;
    fn subject(&self) -> Option<Uuid>;
}

pub trait Event: Serialize {
    fn event_type_version(&self) -> &str;
    fn event_type(&self) -> &str;
    fn event_source(&self) -> &str;
    fn subject(&self) -> Option<Uuid>;
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

pub trait AggregateState {
    fn generation(&self) -> u64;
}

/// A Result where failure is an event sourcing error
pub type Result<T> = std::result::Result<T, Error>;

pub trait Aggregate
{

    type Event: Event;
    type Command : Command;
    type State: AggregateState + Clone + Default;


    fn get_state_for_subject(&self, subject: Uuid) -> Self::State;

    fn get_events_for_subject(&self, subject: Uuid, generation: u64) -> &[Self::Event];

    fn save_events(&self, events : Vec<Self::Event>);

    fn apply_command(&self, cmd: &Self::Command) -> Result<()> {
        if !cmd.is_valid() {
            return Err(Error { kind: Kind::CommandFailure("command is not valid".to_string()) });
        }

        let state = match cmd.subject() {
            Some(subject) => {
                let state =self.get_state_for_subject( subject);
                let old_events = self.get_events_for_subject(subject, state.generation());

                Self::apply_all(&state, old_events).unwrap()

            },
            None => {
                Self::State::default()
            }
        };

        match Self::handle_command(&state, cmd){
            Err(err) => {
                Err(err)
            }
            Ok(events) => {
                self.save_events(events);
                Ok(())
            }
        }
    }

    fn apply_event(state: &Self::State, evt: &Self::Event) -> Result<Self::State>;
    fn handle_command(state: &Self::State, cmd: &Self::Command) -> Result<Vec<Self::Event>>;
    fn apply_all(state: &Self::State, evts: &[Self::Event]) -> Result<Self::State> {
        Ok(evts.iter().fold(state.clone(), |acc_state, event| {
            Self::apply_event(&acc_state, event).unwrap()
        }))
    }
}