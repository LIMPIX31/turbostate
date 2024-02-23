use crate::Engine;
use crate::Flow;

#[derive(Debug, Default)]
pub struct Machine<E: Engine> {
	engine: E,
	state: E::State,
}

#[derive(Debug)]
pub struct Destructed<E: Engine> {
	pub engine: E,
	pub state: E::State,
}

impl<E: Engine> Machine<E> {
	#[allow(dead_code)]
	fn new(engine: E, state: E::State) -> Self {
		Self { engine, state }
	}

	pub async fn fire(&mut self, event: E::Event) -> Result<(), E::Error> {
		match self.engine.next(self.state.clone(), event).await {
			Flow::Pass => (),
			Flow::Transition(new_state) => self.state = new_state,
			Flow::Failure(err) => Err(err)?,
		};

		Ok(())
	}

	pub fn state(&self) -> &E::State {
		&self.state
	}

	pub fn engine(&self) -> &E {
		&self.engine
	}
}

impl<E: Engine> Machine<E>
where
	E::State: Default,
{
	#[allow(dead_code)]
	fn new_default_state(engine: E) -> Self {
		Self {
			engine,
			state: E::State::default(),
		}
	}
}

impl<E: Engine> Machine<E>
where
	E: Default,
{
	#[allow(dead_code)]
	fn new_default_engine(state: E::State) -> Self {
		Self {
			engine: E::default(),
			state,
		}
	}
}
