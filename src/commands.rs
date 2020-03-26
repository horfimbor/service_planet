use planet_interface::PublicCommands;
use event_manager::events::{GenericEvent, Metadata};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) enum PrivateCommands {
    Census,
    Create,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) enum PlanetCommandData {
    Public(PublicCommands),
    Private(PrivateCommands),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct PlanetCommand {
    metadata: Metadata,
    data: PlanetCommandData,
}

impl GenericEvent for PlanetCommand {
    type Payload = PlanetCommandData;

    fn new(metadata: Metadata, payload: Self::Payload) -> Self {
        PlanetCommand{
            metadata,
            data:payload,
        }
    }

    fn get_metadata(&self) -> Metadata {
        self.metadata.clone()
    }

    fn get_event_type(&self) -> String {
        match self.data.clone() {
            PlanetCommandData::Public(public) => {
                match public {
                    PublicCommands::ChangePopulation{..} => {
                        "ChangePopulation".to_string()
                    }
                }
            }
            PlanetCommandData::Private(private) => {
                match private {
                    PrivateCommands::Census => {
                        "Census".to_string()
                    }
                    PrivateCommands::Create => {
                        "Create".to_string()
                    }
                }
            }
        }
    }

    fn is_command(&self) -> bool {
        true
    }

    fn get_payload(&self) -> Self::Payload {
        self.data.clone()
    }
}
