use std::future::Future;

pub trait IntoTransition<T, E> {
	fn into_transition(self) -> Result<T, E>;
}

impl<T, E, I> IntoTransition<T, I> for Result<T, E> where E: Into<I> {
	fn into_transition(self) -> Result<T, I> {
		self.map_err(Into::into)
	}
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
