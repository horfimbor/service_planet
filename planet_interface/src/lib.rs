#[macro_use]
extern crate serde_derive;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PublicCommands {
    ChangePopulation { pop_change: i64 },
}