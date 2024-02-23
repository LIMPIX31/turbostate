use std::future::Future;
use crate::Flow;

pub trait Engine {
	type Flow;
	type State: Clone;
	type Event;
	type Error;

	#[allow(unused)]
	fn next(&mut self, state: Self::State, event: Self::Event) -> impl Future<Output = Flow<Self::State, Self::Error>>;
}
