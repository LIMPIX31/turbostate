use crate::Engine;
use crate::AsyncEngine;
use anyhow::Result;

mod switch {
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
		fn a_switch(&self) {
			B
		}

		#[branch((B, Switch))]
		fn b_switch(&self) {
			A
		}
	}
}

use switch::Engine as SwitchEngine;

#[test]
fn switch() -> Result<()> {
	use switch::State;
	use switch::Event;

	let mut engine = SwitchEngine;
	let mut state = State::default();
	//               A -> B         B -> A         A -> B
	let events = [Event::Switch, Event::Switch, Event::Switch];

	for event in events {
		state = engine.next(state, event)?;
	}

	assert!(matches!(state, State::B));

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

	use State::*;
	use Event::*;

	#[engine(async)]
	impl Engine {
		#[branch((Idle, Dial))]
		async fn idle_dial(&self) {
			Dialing
		}

		#[branch((Dialing, Reject))]
		async fn dialing_reject(&self) {
			Idle
		}

		#[branch((Dialing, Answer))]
		async fn dialing_answer(&mut self) {
			self.calls_made += 1;
			Connected
		}

		#[branch((Connected, HangUp))]
		async fn connected_hangup(&self) {
			Disconnected
		}

		#[branch((Disconnected, Reset))]
		async fn disconencted_reset(&self) {
			Idle
		}

		#[branch((Idle, IncomingCall))]
		async fn idle_incoming_call(&self) {
			Ringing
		}

		#[branch((Ringing, Reject))]
		async fn ringing_reject(&self) {
			Idle
		}

		#[branch((Ringing, Answer))]
		async fn ringing_answer(&mut self) {
			self.calls_received += 1;
			Connected
		}

		#[branch(_)]
		async fn rest(&self) {
			Err(Error::InvalidTransition)
		}
	}
}

use call::Engine as CallEngine;

#[tokio::test]
async fn call() -> Result<()> {
	use call::State;
	use call::Event;

	let script = [
		Event::Dial,
		Event::Answer,
		Event::HangUp,
		Event::Reset,
		Event::IncomingCall,
		Event::Answer,
		Event::HangUp,
	];

	let mut engine = CallEngine::default();
	let mut state = State::default();

	for event in script {
		state = engine.next(state, event).await?;
	}

	assert!(matches!(state, State::Disconnected));
	assert_eq!(engine.calls_made, 1);
	assert_eq!(engine.calls_received, 1);

	Ok(())
}
