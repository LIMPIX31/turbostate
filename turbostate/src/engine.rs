use std::future::Future;

pub trait IntoTransition<T, E> {
	fn into_transition(self) -> Result<T, E>;
}

pub trait Engine {
	type State;
	type Event;
	type Error;

	fn next(&mut self, state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error>;
}

pub trait AsyncEngine {
	type State;
	type Event;
	type Error;

	fn next(&mut self, state: Self::State, event: Self::Event) -> impl Future<Output = Result<Self::State, Self::Error>>;
}
