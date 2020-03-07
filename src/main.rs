#[macro_use]
extern crate eventsourcing_derive;

#[macro_use]
extern crate serde_derive;

use eventsourcing::eventstore::EventStore;

use eventsourcing::{eventstore::MemoryEventStore, prelude::*, Result};

use planet_interface::PublicCommands;

mod commands;

use commands::PrivateCommands;

const DOMAIN_VERSION: &str = "1.0";


#[derive(Debug, Clone)]
struct PlanetData {
    generation: u64,
    pop: u64,
}

impl Default for PlanetData {
    fn default() -> Self {
        PlanetData {
            generation: 0,
            pop: 0,
        }
    }
}

impl AggregateState for PlanetData {
    fn generation(&self) -> u64 {
        self.generation
    }
}

#[event_type_version(DOMAIN_VERSION)]
#[event_source("events://github.com/pholactery/eventsourcing/samples/location")]
#[derive(Serialize, Deserialize, Debug, Clone, Event)]
enum PlanetEvent {
    PopulationUpdated { pop: u64 },
}


enum PlanetCommands {
    Public(PublicCommands),
    Private(PrivateCommands),
}

struct Planet;

impl Aggregate for Planet {
    type Event = PlanetEvent;
    type Command = PlanetCommands;
    type State = PlanetData;

    fn apply_event(state: &Self::State, evt: &Self::Event) -> Result<Self::State> {
        let planete = match *evt {
            PlanetEvent::PopulationUpdated { pop } => {
                PlanetData {
                    pop,
                    generation: state.generation + 1,
                }
            }
        };
        Ok(planete)
    }

    fn handle_command(state: &Self::State, cmd: &Self::Command) -> Result<Vec<Self::Event>> {
        let evt = match cmd {
            PlanetCommands::Private(private_command) => {
                match private_command {
                    PrivateCommands::Census => {
                        PlanetEvent::PopulationUpdated { pop: state.pop +12 }
                    }
                }
            }
            PlanetCommands::Public(public_command) => {
                match public_command {
                    PublicCommands::ChangePopulation { pop_change } => {
                        if *pop_change > 0 {
                            PlanetEvent::PopulationUpdated { pop: state.pop + *pop_change as u64 }
                        } else {
                            let killed = -pop_change as u64;
                            if killed > state.pop {
                                PlanetEvent::PopulationUpdated { pop: 0 }
                            } else {
                                PlanetEvent::PopulationUpdated { pop: state.pop - killed }
                            }
                        }
                    }
                }
            }
        };

        Ok(vec![evt])
    }
}


fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let planet_store = MemoryEventStore::new();

    let default_planet = PlanetData::default();
    println!("default_planet: {:?}", default_planet);

    let add_colon = PlanetCommands::Public(PublicCommands::ChangePopulation { pop_change: 1000 })  ;
    let kill_colon = PlanetCommands::Public( PublicCommands::ChangePopulation { pop_change: -570 });

    let events_add_colon = Planet::handle_command(&default_planet, &add_colon)?;
    planet_store.append(events_add_colon[0].clone(), "planet")?;
    let with_colon = Planet::apply_all(&default_planet, &events_add_colon)?;
    println!("with_colon: {:?}", with_colon);

    let events_kill_colon_one = Planet::handle_command(&with_colon, &kill_colon)?;
    planet_store.append(events_kill_colon_one[0].clone(), "planet")?;
    let with_kill_one = Planet::apply_all(&with_colon, &events_kill_colon_one)?;
    println!("with_kill_one: {:?}", with_kill_one);

    let events_kill_colon_two = Planet::handle_command(&with_kill_one, &kill_colon)?;
    planet_store.append(events_kill_colon_two[0].clone(), "planet")?;
    let with_kill_two = Planet::apply_all(&with_colon, &events_kill_colon_two)?;
    println!("with_kill_two: {:?}", with_kill_two);

    println!(
        "all events - {:#?}",
        planet_store.get_all("planetevent.populationupdated")
    );

    Ok(())
}