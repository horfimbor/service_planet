

// use planet_interface::PublicEvents;
use uuid::Uuid;
use event_manager::{Event};
use super::DOMAIN_VERSION;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) enum PlanetEventData {
    PopulationUpdated { pop: u64 },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct PlanetEvent{
    subject: Option<Uuid>,
    data : PlanetEventData,
}



impl Event for PlanetEvent{
    type Data = PlanetEventData;

    fn new(subject: Option<Uuid>, data: Self::Data) -> Self {
        PlanetEvent{
            subject,
            data
        }
    }

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
        self.subject
    }

    fn data(&self) -> &Self::Data {
        &self.data
    }
}

