#[macro_use]
extern crate serde_derive;

use eventstore::{Connection};
use futures::executor::block_on;
use event_manager::events::{GenericEvent, Metadata};
use event_manager::{Aggregate, Result as EmResult};
use uuid::Uuid;
use serde_json::json;
use planet_interface::PublicCommands;

mod commands;

use commands::{PlanetCommand, PlanetCommandData, PrivateCommands};

mod events;

use events::{PlanetEvent, PlanetEventData};

mod aggregate;

use aggregate::PlanetData;
use std::str;
use futures::TryStreamExt;

const AGGREGATE_PREFIX:&str = "planet_";

struct Planet<'a> {
    event_store: &'a Connection
}

impl<'a> Planet<'a> {
    fn new (conn:  &'a Connection) -> Self {
        Planet {
            event_store: conn
        }
    }

    pub async fn save_event(&self, event: &<Planet<'_> as Aggregate>::Event) -> Result<(), Box<dyn std::error::Error>> {

        let stream_id = format!("{}{}", AGGREGATE_PREFIX, event.get_aggregate_id());

        let payload = json!( event.get_payload() );
        let metadata =  json!(event.get_metadata());

        let data = eventstore::EventData::json(event.get_event_type(), payload).unwrap();

        let data = data.metadata_as_json(metadata);

        let result = self.event_store
            .write_events(stream_id)
            .push_event(data)
            .execute()
            .await?;

        println!("Write response: {:?}", result);

        Ok(())
    }
}

impl Aggregate for Planet<'_> {
    type Event = PlanetEvent;
    type Command = PlanetCommand;
    type State = PlanetData;

    fn load_state(&self, _subject: Uuid) -> Self::State {
        Self::State::default()
    }

    fn save_state(&self, _state: Self::State) -> EmResult<()> {
        Ok(())
    }

    fn load_events(&self, subject: Uuid, _generation: u64) -> Vec<Self::Event> {

        let mut vec = Vec::new();

        block_on( async {
            let stream_id = format!("{}{}", AGGREGATE_PREFIX, subject);

            let mut stream = self.event_store
                .read_stream(stream_id.as_str())
                .start_from_beginning()
                .max_count(1)
                .iterate_over();

            while let Some(event) = stream.try_next().await.unwrap() {
                let event = event.get_original_event();

               let data : PlanetEventData = serde_json::from_str( str::from_utf8(&event.data).unwrap()).unwrap();
               let metadata : Metadata = serde_json::from_str( str::from_utf8(&event.metadata).unwrap()).unwrap();


                let event = PlanetEvent::new(metadata, data);

                vec.push( event);
            }

        });
        vec
    }

    fn save_events(&self, events: Vec<Self::Event>) -> EmResult<()> {
        for event in &events {
            let t = block_on(self.save_event(event));

            match t {
                Ok(_0) => {}
                _ => {
                    print!("ERRORO");
                }
            }
        }
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

    let cmd = PlanetCommand::new(meta_cmd, PlanetCommandData::Private( PrivateCommands::Create ) );
    planet.handle_command(&cmd);

    let meta_cmd = Metadata::new_for_command(aggregate_id);

    let cmd = PlanetCommand::new(meta_cmd, PlanetCommandData::Public( PublicCommands::ChangePopulation {pop_change: 20} ) );
    planet.handle_command(&cmd);

    let meta_cmd = Metadata::new_for_command(aggregate_id);

    let cmd = PlanetCommand::new(meta_cmd, PlanetCommandData::Public( PublicCommands::ChangePopulation {pop_change: -1000} ) );
    planet.handle_command(&cmd);


    Ok(())
}
