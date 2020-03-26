#[macro_use]
extern crate serde_derive;

use eventstore::{Connection, EventData};
use futures::executor::block_on;
use event_manager::events::{GenericEvent, Metadata};
use event_manager::{Aggregate, AggregateState, Result as EmResult};
use uuid::Uuid;
use serde_json::json;
use planet_interface::PublicCommands;

mod commands;

use commands::{PlanetCommand, PlanetCommandData, PrivateCommands};

mod events;

use events::{PlanetEvent, PlanetEventData};

mod aggregate;

use aggregate::PlanetData;
use std::net::SocketAddr;
use std::process::Command;
use std::error::Error;


struct Planet {
    event_store: SocketAddr
}

impl Planet {
    fn new(socket: SocketAddr) -> Self {
        Planet {
            event_store: socket
        }
    }

    async fn save_event(&self, event: &<Planet as Aggregate>::Event) -> Result<(), Box<dyn std::error::Error>> {
        let connection = Connection::builder().single_node_connection(self.event_store).await;

        //let payload = json!(event.get_payload());
//
        // let event_storable = EventData::json(event.get_event_type(), payload.clone())?;
        //
        // let event_storable = event_storable.metadata_as_json(json!(event.get_metadata()));

        let payload = json!({
            "is_rust_a_nice_language": true,
        });

        let event_storable = EventData::json("language-poll", payload)?;

        println!("try");

        let result = connection
            .write_events("AZERRTY")
            .push_event(event_storable)
            .execute().await?;

        println!("{:?}", result);

        Ok(())
    }
}

impl Aggregate for Planet {
    type Event = PlanetEvent;
    type Command = PlanetCommand;
    type State = PlanetData;

    fn load_state(&self, subject: Uuid) -> Self::State {
        Self::State::default()
    }

    fn save_state(&self, state: Self::State) -> EmResult<()> {
        Ok(())
    }

    fn load_events(&self, subject: Uuid, generation: u64) -> Vec<Self::Event> {
        let vec = Vec::new();
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    block_on(async {
        use std::env;



        let endpoint = "172.28.1.1:1113".parse().unwrap();
        println!("Connection string: {}", endpoint);

        let connection = eventstore::Connection::builder()
            .with_default_user(eventstore::Credentials::new("admin", "changeit"))
            .single_node_connection(endpoint)
            .await;
        println!("connected to : {}", endpoint);

        let stream_id = "TEST";

        let payload = json!({
                "event_index": "bla",
            });

        let data = eventstore::EventData::json("event_test", payload).unwrap();


        let result = connection
            .write_events(stream_id)
            .push_event(data)
            .execute()
            .await?;

        println!("Write response: {:?}", result);

        connection.shutdown().await;

        Ok(()) as Result<(), Box<dyn Error>>
    })
        .unwrap();


// async fn main() -> EmResult<()> {
    // let planet = Planet::new("127.0.0.1:1113".parse().unwrap());
    //
    // let planete_id = Uuid::new_v4();
    //
    // let create_planet = PlanetCommand::new(
    //     Metadata::new_for_command(planete_id),
    //     PlanetCommandData::Private(PrivateCommands::Create),
    // );
    //
    // planet.handle_command(&create_planet).unwrap();

    // let default_planet = PlanetData::default();
    // println!("default_planet: {:?}", default_planet);
    //
    // let subject = Some(Uuid::new_v4());
    //
    // let add_colon = PlanetCommand::new(subject, PlanetCommandData::Public(PublicCommands::ChangePopulation { pop_change: 1000 }));
    // // let kill_colon = PlanetCommandData::Public(PublicCommands::ChangePopulation { pop_change: -570 });
    //
    // let events_add_colon = Planet::handle_command(&default_planet, &add_colon).unwrap();
    //
    // println!("events - {}",  json!(events_add_colon));


    // // Do something productive with the result.
    // println!("{:?}", result);

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
