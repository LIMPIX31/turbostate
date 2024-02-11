//! `turbostate` is a library for building state machines in Rust

#![cfg_attr(feature = "from_residual", feature(try_trait_v2))]

#[cfg(test)]
mod tests;

#[cfg(feature = "async")]
use std::future::Future;
#[cfg(not(feature = "async"))]
use std::sync::Mutex;
#[cfg(feature = "async")]
use tokio::sync::Mutex;

/// `Flow` represents the possible outcomes of state transitions in the state
/// machine.
#[derive(Debug)]
pub enum Flow<T, E, B> {
	/// Skip to the next step without changing the state.
	Pass,
	/// Transition to a new state.
	Transition(T),
	/// Jump to another branch within the same event, specifying a new state and
	/// event.
	Slide(T, B),
	/// Raise an error if an error occurs during the transition.
	Failure(E),
}

#[cfg(feature = "from_residual")]
impl<T, E, B, X, R> std::ops::FromResidual<Result<T, E>> for Flow<X, R, B>
where
	E: Into<R>,
{
	fn from_residual(residual: Result<T, E>) -> Self {
		if let Err(err) = residual {
			Self::Failure(err.into())
		} else {
			unreachable!()
		}
	}
}

/// `Engine` is a trait that must be implemented on the state machine.
///
/// This trait defines the behavior of the state machine, including state
/// transitions based on events, error handling, and shared data management.
pub trait Engine {
	/// Represents the enum of possible states.
	type State;
	/// Represents the type of events that drive state transitions.
	type Event;
	/// Represents the type of errors that can occur during state transitions.
	type Error;
	/// Represents any shared data that is accessible by all states.
	type Shared;

	/// Advances the state machine based on the current state, incoming event, and
	/// shared data.
	#[cfg(feature = "async")]
	#[allow(unused)]
	fn next(
		state: &Self::State,
		event: Self::Event,
		shared: &mut Self::Shared,
	) -> impl Future<Output = Flow<Self::State, Self::Error, Self::Event>> + Send {
		async move { Flow::Pass }
	}

	/// Advances the state machine based on the current state, incoming event, and
	/// shared data.
	#[cfg(not(feature = "async"))]
	#[allow(unused)]
	fn next(
		state: &Self::State,
		event: Self::Event,
		shared: &mut Self::Shared,
	) -> Flow<Self::State, Self::Error, Self::Event> {
		Flow::Pass
	}
}

#[derive(Debug, Default)]
struct Store<T, S> {
	state: Mutex<T>,
	shared: Mutex<S>,
}

impl<T, S> Store<T, S> {
	pub fn new(state: T, shared: S) -> Self {
		Self {
			state: Mutex::new(state),
			shared: Mutex::new(shared),
		}
	}
}

#[derive(Debug)]
pub struct DestructedMachine<T, S> {
	pub state: T,
	pub shared: S,
}

/// `Machine` is a struct that encapsulates the state and shared data of the
/// state machine, providing methods to advance the state based on events.
#[derive(Debug, Default)]
pub struct Machine<T: Engine> {
	store: Store<T::State, T::Shared>,
}

impl<T: Engine> Machine<T> {
	/// Creates a new `Machine` with the initial state and default shared data.
	pub fn new(initial: T::State) -> Self
	where
		T::Shared: Default,
	{
		Self {
			store: Store::new(initial, T::Shared::default()),
		}
	}

	/// Creates a new `Machine` with the initial state and specified shared data.
	pub fn new_shared(initial: T::State, shared: T::Shared) -> Self {
		Self {
			store: Store::new(initial, shared),
		}
	}

	/// Creates a new `Machine` with the default state and specified shared data.
	pub fn default_shared(shared: T::Shared) -> Self
	where
		T::State: Default,
	{
		Self {
			store: Store::new(T::State::default(), shared),
		}
	}

	/// Destructs machine and returns inner state
	#[cfg(feature = "async")]
	pub fn destruct(self) -> DestructedMachine<T::State, T::Shared> {
		DestructedMachine {
			state: self.store.state.into_inner(),
			shared: self.store.shared.into_inner(),
		}
	}

	/// Destructs machine and returns inner state
	#[cfg(not(feature = "async"))]
	pub fn destruct(self) -> DestructedMachine<T::State, T::Shared> {
		DestructedMachine {
			state: self.store.state.into_inner().unwrap(),
			shared: self.store.shared.into_inner().unwrap(),
		}
	}

	#[cfg(feature = "async")]
	async fn set_state(&self, new_state: T::State) {
		let mut state = self.store.state.lock().await;
		*state = new_state;
	}

	#[cfg(not(feature = "async"))]
	fn set_state(&self, new_state: T::State) {
		let mut state = self.store.state.lock().unwrap();
		*state = new_state;
	}

	/// Fires the specified event on the state machine to advance the state
	/// asynchronously.
	#[cfg(feature = "async")]
	#[async_recursion::async_recursion]
	pub async fn fire(&self, event: T::Event) -> Result<(), T::Error>
	where
		T::Event: Send,
		T::Shared: Send,
		T::State: Send,
		T::Error: Send,
	{
		let result = {
			let state = self.store.state.lock().await;
			let mut shared = self.store.shared.lock().await;
			T::next(&state, event, &mut shared).await
		};

		match result {
			Flow::Pass => Ok(()),
			Flow::Transition(new_state) => {
				self.set_state(new_state).await;
				Ok(())
			}
			Flow::Slide(new_state, event) => {
				self.set_state(new_state).await;
				self.fire(event).await
			}
			Flow::Failure(err) => Err(err),
		}
	}

	/// Fires the specified event on the state machine to advance the state.
	#[cfg(not(feature = "async"))]
	pub fn fire(&self, event: T::Event) -> Result<(), T::Error> {
		let result = {
			let state = self.store.state.lock().unwrap();
			let mut shared = self.store.shared.lock().unwrap();
			T::next(&state, event, &mut shared)
		};

		match result {
			Flow::Pass => Ok(()),
			Flow::Transition(new_state) => {
				self.set_state(new_state);
				Ok(())
			}
			Flow::Slide(new_state, event) => {
				self.set_state(new_state);
				self.fire(event)
			}
			Flow::Failure(err) => Err(err),
		}
	}
}
