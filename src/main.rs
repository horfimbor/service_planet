
#[macro_use]
extern crate serde_derive;

use eventstore::{Connection, EventData};
use futures::Future;
use planet_interface::PublicCommands;
use event_manager::cloudevents::CloudEvent;
use event_manager::{Aggregate, AggregateState, Command, Event, Error};
use uuid::Uuid;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
enum PlanetEventData {
    PopulationUpdated { pop: u64 },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PlanetEvent{
    subject: Uuid,
    data : PlanetEventData,
}


impl Event for PlanetEvent{
    fn event_type_version(&self) -> &str {
        "0.1.0"
    }

    fn event_type(&self) -> &str {
        "planet_event"
    }

    fn event_source(&self) -> &str {
        "https://github.com/horfimbor/service_planet"
    }

    fn subject(&self) -> Option<Uuid> {
        Some(self.subject)
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
enum PlanetCommandData {
    Public(PublicCommands),
    Private(PrivateCommands),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PlanetCommand{
    subject: Uuid,
    data : PlanetCommandData,
}

impl Command for PlanetCommand{
    fn event_type_version(&self) -> &str {
        "0.1.0"
    }

    fn event_type(&self) -> &str {
        "planet_command"
    }

    fn event_source(&self) -> &str {
        "https://github.com/horfimbor/service_planet"
    }

    fn is_valid(&self) -> bool {
        return true
    }

    fn subject(&self) -> Option<Uuid> {
        return Some(self.subject)
    }
}

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
        let planet = match evt.data {
            PlanetEventData::PopulationUpdated { pop } => {
                PlanetData {
                    pop,
                    generation: state.generation + 1,
                }
            }
        };
        Ok(planet)
    }

    fn handle_command(state: &Self::State, cmd: &Self::Command) -> Result<Vec<Self::Event>, Error> {
        let data = match &cmd.data {
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

        let event = PlanetEvent{
            data,
            subject : cmd.subject
        };

        Ok(vec![event])
    }
}


fn main() -> std::result::Result<(), Error > {
    let default_planet = PlanetData::default();
    println!("default_planet: {:?}", default_planet);

    let subject = Uuid::new_v4();

    let add_colon = PlanetCommand{subject, data:PlanetCommandData::Public(PublicCommands::ChangePopulation { pop_change: 1000 })};
    // let kill_colon = PlanetCommandData::Public(PublicCommands::ChangePopulation { pop_change: -570 });

    let events_add_colon = Planet::handle_command(&default_planet, &add_colon)?;

    println!("events - {:#?}", CloudEvent::from(events_add_colon[0].clone()));

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