

pub trait AggregateState  : Clone + Default {
    fn generation(&self) -> u64;
}