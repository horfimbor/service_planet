
use event_manager::events::{GenericEvent, Metadata};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) enum PlanetEventData {
    PopulationCreate { pop: u64 },
    PopulationIncrease { pop: u64 },
    PopulationDecrease { pop: u64 },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct PlanetEvent{
    metadata : Metadata,
    data : PlanetEventData,
}

impl GenericEvent for PlanetEvent{
    type Payload = PlanetEventData;

    fn new(metadata: Metadata, payload: Self::Payload) -> Self {
        PlanetEvent{
            metadata,
            data:payload,
        }
    }

    fn get_metadata(&self) -> Metadata {
        self.metadata.clone()
    }

    fn get_event_type(&self) -> String {
        match self.data {
            PlanetEventData::PopulationCreate { .. } => {
                "PopulationCreate".to_string()
            },
            PlanetEventData::PopulationIncrease { .. } => {
                "PopulationIncrease".to_string()
            },
            PlanetEventData::PopulationDecrease { .. } => {
                "PopulationDecrease".to_string()
            },
        }
    }

    fn is_command(&self) -> bool {
        false
    }

    fn get_payload(&self) -> Self::Payload {
        self.data.clone()
    }
}

