#[macro_use]
extern crate serde_derive;

use eventstore::Connection;
use event_manager::events::{GenericEvent, Metadata};
use event_manager::{Aggregate, Result as EmResult};
use uuid::Uuid;
use planet_interface::PublicCommands;

mod commands;

use commands::{PlanetCommand, PlanetCommandData, PrivateCommands};

mod events;

use events::{PlanetEvent, PlanetEventData};

mod aggregate;

use aggregate::PlanetData;
use std::str;

const AGGREGATE_PREFIX: &str = "planet_";

struct Planet<'a> {
    event_store: &'a Connection
}

impl<'a> Planet<'a> {
    fn new(conn: &'a Connection) -> Self {
        Planet {
            event_store: conn
        }
    }
}

impl Aggregate for Planet<'_> {
    type Event = PlanetEvent;
    type Command = PlanetCommand;
    type State = PlanetData;

    fn get_aggregate_prefix() -> &'static str {
        AGGREGATE_PREFIX
    }

    fn get_connection(&self) -> &Connection {
        self.event_store
    }

    fn load_state(&self, _subject: Uuid) -> Self::State {
        Self::State::default()
    }

    fn save_state(&self, _state: Self::State) -> EmResult<()> {
        Ok(())
    }

    fn apply_event(state: &Self::State, evt: &Self::Event) -> EmResult<Self::State> {
        let mut planet = match evt.get_payload() {
            PlanetEventData::PopulationCreate { pop } => {
                let mut new_state = state.clone();
                new_state.pop = pop;
                new_state
            }
            PlanetEventData::PopulationIncrease { pop } => {
                let mut new_state = state.clone();
                new_state.pop += pop;
                new_state
            }
            PlanetEventData::PopulationDecrease { pop } => {
                let mut new_state = state.clone();
                new_state.pop -= pop;
                new_state
            }
        };
        planet.generation += 1;
        Ok(planet)
    }

    fn apply_command(state: &Self::State, cmd: &Self::Command) -> EmResult<Vec<Self::Event>> {
        let data = match &cmd.get_payload() {
            PlanetCommandData::Private(private_command) => {
                match private_command {
                    PrivateCommands::Census => {
                        PlanetEventData::PopulationIncrease { pop: 12 }
                    }
                    PrivateCommands::Create => {
                        PlanetEventData::PopulationCreate { pop: 500 }
                    }
                }
            }
            PlanetCommandData::Public(public_command) => {
                match public_command {
                    PublicCommands::ChangePopulation { pop_change } => {
                        if *pop_change > 0 {
                            PlanetEventData::PopulationIncrease { pop: *pop_change as u64 }
                        } else {
                            let decrease = -pop_change as u64;
                            if decrease > state.pop {
                                PlanetEventData::PopulationDecrease { pop: state.pop }
                            } else {
                                PlanetEventData::PopulationDecrease { pop: decrease }
                            }
                        }
                    }
                }
            }
        };

        let event = PlanetEvent::new(
            Metadata::new_for_event(cmd.get_metadata()),
            data,
        );

        Ok(vec![event])
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = "127.0.0.1:1113".parse().unwrap();
    let connection = eventstore::Connection::builder()
        .with_default_user(eventstore::Credentials::new("admin", "changeit"))
        .single_node_connection(endpoint)
        .await;

    let planet = Planet::new(&connection);

    let aggregate_id = Uuid::new_v4();

    let meta_cmd = Metadata::new_for_command(aggregate_id);

    let cmd = PlanetCommand::new(meta_cmd, PlanetCommandData::Private(PrivateCommands::Create));
    planet.handle_command(&cmd);

    let meta_cmd = Metadata::new_for_command(aggregate_id);

    let cmd = PlanetCommand::new(meta_cmd, PlanetCommandData::Public(PublicCommands::ChangePopulation { pop_change: 20 }));
    planet.handle_command(&cmd);

    let meta_cmd = Metadata::new_for_command(aggregate_id);

    let cmd = PlanetCommand::new(meta_cmd, PlanetCommandData::Public(PublicCommands::ChangePopulation { pop_change: -1000 }));
    planet.handle_command(&cmd);


    Ok(())
}
