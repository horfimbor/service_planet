
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate tokio;

use eventstore::{Connection, EventData};
use futures::Future;
use event_manager::cloudevents::CloudEvent;
use event_manager::{Aggregate, AggregateState, Command, Event, Error};
use uuid::Uuid;
use serde_json::json;
use planet_interface::PublicCommands;

mod commands;
use commands::{PlanetCommand,PlanetCommandData,PrivateCommands};

mod events;
use events::{PlanetEvent,PlanetEventData};

mod aggregate;
use aggregate::PlanetData;

const DOMAIN_VERSION: &str = "1.0";


struct Planet;

impl Aggregate for Planet {
    type Event = PlanetEvent;
    type Command = PlanetCommand;
    type State = PlanetData;

    fn get_state_for_subject(&self, _subject: Uuid) -> Self::State {
        return Self::State::default()
    }

    fn get_events_for_subject(&self, subject: Uuid, generation: u64) -> &[Self::Event] {
        unimplemented!()
    }

    fn save_events(&self, events: Vec<Self::Event>) {
        unimplemented!()
    }

    fn apply_event(state: &Self::State, evt: &Self::Event) -> Result<Self::State, Error> {
        let planet = match evt.data() {
            PlanetEventData::PopulationUpdated { pop } => {
                PlanetData {
                    pop: *pop,
                    generation: state.generation() + 1,
                }
            }
        };
        Ok(planet)
    }

    fn handle_command(state: &Self::State, cmd: &Self::Command) -> Result<Vec<Self::Event>, Error> {
        let data = match &cmd.data() {
            PlanetCommandData::Private(private_command) => {
                match private_command {
                    PrivateCommands::Census => {
                        PlanetEventData::PopulationUpdated { pop: state.pop + 12 }
                    }
                }
            }
            PlanetCommandData::Public(public_command) => {
                match public_command {
                    PublicCommands::ChangePopulation { pop_change } => {
                        if *pop_change > 0 {
                            PlanetEventData::PopulationUpdated { pop: state.pop + *pop_change as u64 }
                        } else {
                            let killed = -pop_change as u64;
                            if killed > state.pop {
                                PlanetEventData::PopulationUpdated { pop: 0 }
                            } else {
                                PlanetEventData::PopulationUpdated { pop: state.pop - killed }
                            }
                        }
                    }
                }
            }
        };

        let event = PlanetEvent::new(cmd.subject(), data);

        Ok(vec![event])
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let default_planet = PlanetData::default();
    println!("default_planet: {:?}", default_planet);

    let subject = Some(Uuid::new_v4());

    let add_colon = PlanetCommand::new(subject, PlanetCommandData::Public(PublicCommands::ChangePopulation { pop_change: 1000 }));
    // let kill_colon = PlanetCommandData::Public(PublicCommands::ChangePopulation { pop_change: -570 });

    let events_add_colon = Planet::handle_command(&default_planet, &add_colon).unwrap();

    println!("events - {}",  json!(CloudEvent::from(events_add_colon[0].clone())));

    let addr = "127.0.0.1:1113".parse()?;
    let evenstore = Connection::builder()
        .single_node_connection(addr)
        .await;

    let payload =  json!(CloudEvent::from(events_add_colon[0].clone()));
    let event =  EventData::json("TEST", payload.clone())?;
    //
    let event =event.metadata_as_json(payload);

    let result = evenstore
        .write_events("TEST-stream")
        .push_event(event)
        .execute()
        .await?;

    // Do something productive with the result.
    println!("{:?}", result);

    // planet_store.append(events_add_colon[0].clone(), "planet")?;
    // let with_colon = Planet::apply_all(&default_planet, &events_add_colon)?;
    // println!("with_colon: {:?}", with_colon);
    //
    // let events_kill_colon_one = Planet::handle_command(&with_colon, &kill_colon)?;
    // planet_store.append(events_kill_colon_one[0].clone(), "planet")?;
    // let with_kill_one = Planet::apply_all(&with_colon, &events_kill_colon_one)?;
    // println!("with_kill_one: {:?}", with_kill_one);
    //
    // let events_kill_colon_two = Planet::handle_command(&with_kill_one, &kill_colon)?;
    // planet_store.append(events_kill_colon_two[0].clone(), "planet")?;
    // let with_kill_two = Planet::apply_all(&with_colon, &events_kill_colon_two)?;
    // println!("with_kill_two: {:?}", with_kill_two);

    // println!(
    //     "all events - {:#?}",
    //     planet_store.get_all("planetevent.populationupdated")
    // );

    Ok(())
}