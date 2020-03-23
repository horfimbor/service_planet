
use planet_interface::PublicCommands;
use uuid::Uuid;
use event_manager::{Command};
use super::DOMAIN_VERSION;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PrivateCommands {
    Census,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PlanetCommandData {
    Public(PublicCommands),
    Private(PrivateCommands),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlanetCommand{
    subject: Option<Uuid>,
    data : PlanetCommandData,
}

impl Command for PlanetCommand{
    type Data = PlanetCommandData;

    fn new(subject: Option<Uuid>, data: Self::Data) -> Self {
        PlanetCommand{
            subject,
            data
        }
    }

    fn event_type_version(&self) -> &str {
       DOMAIN_VERSION
    }

    fn event_type(&self) -> &str {
        "planet_command"
    }

    fn event_source(&self) -> &str {
        "https://github.com/horfimbor/service_planet"
    }

    fn subject(&self) -> Option<Uuid> {
        return self.subject
    }

    fn data(&self) -> &PlanetCommandData {
        return &self.data
    }

    fn is_valid(&self) -> bool {
        return true
    }
}
