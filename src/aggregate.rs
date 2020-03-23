

use event_manager::{AggregateState};

#[derive(Debug, Clone)]
pub(crate) struct PlanetData {
    pub generation: u64,
    pub pop: u64,
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
