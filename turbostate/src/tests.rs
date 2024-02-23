use crate::machine::Machine;
use anyhow::Result;

mod switch {
	use crate::Flow::*;
	use std::convert::Infallible;
	use turbostate_macros::engine;

	#[derive(Debug, Clone, Default, PartialEq, Eq)]
	pub enum State {
		#[default]
		A,
		B,
	}

	#[derive(Debug)]
	pub enum Event {
		Switch,
	}

	pub type Error = Infallible;

	#[derive(Debug, Default)]
	pub struct Engine;

	use State::*;
	use Event::*;

	#[engine]
	impl Engine {
		#[branch((A, Switch))]
		async fn a_to_b(&mut self) {
			Transition(B)
		}

		#[branch((B, Switch))]
		async fn b_to_a(&mut self) {
			Transition(A)
		}
	}
}

use switch::Engine as SwitchEngine;

#[tokio::test]
async fn switch() -> Result<()> {
	use switch::State;
	use switch::Event;

	let mut machine = Machine::<SwitchEngine>::default();
	assert!(matches!(machine.state(), State::A));
	machine.fire(Event::Switch).await?;
	assert!(matches!(machine.state(), State::B));
	machine.fire(Event::Switch).await?;
	assert!(matches!(machine.state(), State::A));

	Ok(())
}

mod call {
	use thiserror::Error;
	use crate::engine;

	#[derive(Debug, Clone, Default, PartialEq, Eq)]
	pub enum State {
		#[default]
		Idle,
		Dialing,
		Ringing,
		Connected,
		Disconnected,
	}

	#[allow(dead_code)]
	#[derive(Debug)]
	pub enum Event {
		Dial,
		IncomingCall,
		Answer,
		Reject,
		HangUp,
		Reset,
	}

	#[derive(Debug, Error)]
	pub enum Error {
		#[error("Invalid transition")]
		InvalidTransition,
	}

	#[derive(Debug, Default)]
	pub struct Engine {
		pub calls_made: u32,
		pub calls_received: u32,
	}

	use crate::Flow::*;

	use State::*;
	use Event::*;

	#[engine]
	impl Engine {
		#[branch((Idle, Dial))]
		async fn idle_dial(&self) {
			Transition(Dialing)
		}

		#[branch((Dialing, Reject))]
		async fn dialing_reject(&self) {
			Transition(Idle)
		}

		#[branch((Dialing, Answer))]
		async fn dialing_answer(&mut self) {
			self.calls_made += 1;
			Transition(Connected)
		}

		#[branch((Connected, HangUp))]
		async fn connected_hangup(&self) {
			Transition(Disconnected)
		}

		#[branch((Disconnected, Reset))]
		async fn disconencted_reset(&self) {
			Transition(Idle)
		}

		#[branch((Idle, IncomingCall))]
		async fn idle_incoming_call(&self) {
			Transition(Ringing)
		}

		#[branch((Ringing, Reject))]
		async fn ringing_reject(&self) {
			Transition(Idle)
		}

		#[branch((Ringing, Answer))]
		async fn ringing_answer(&mut self) {
			self.calls_received += 1;
			Transition(Connected)
		}

		#[branch(_)]
		async fn rest(&self) {
			// With `from_residual` feature enabled you can do `Err(InvalidTransition)?`
			Failure(Error::InvalidTransition)
		}
	}
}

use call::Engine as CallEngine;

#[tokio::test]
async fn call() -> Result<()> {
	use call::State;
	use call::Event;

	let mut machine = Machine::<CallEngine>::default();

	machine.fire(Event::Dial).await?;
	machine.fire(Event::Answer).await?;
	machine.fire(Event::HangUp).await?;

	machine.fire(Event::Reset).await?;
	machine.fire(Event::IncomingCall).await?;
	machine.fire(Event::Answer).await?;
	machine.fire(Event::HangUp).await?;

	assert!(matches!(machine.state(), State::Disconnected));
	assert_eq!(machine.engine().calls_made, 1);
	assert_eq!(machine.engine().calls_received, 1);

	Ok(())
}
