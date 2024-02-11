# Turbostate
######  I just needed a state machine, so I wrote this.

## By example
```rust
use turbostate::{Engine, Flow, Machine};

#[derive(Debug, Default, PartialEq, Eq)]
enum State {
	#[default]
	Idle,
	Dialing,
	Ringing,
	Connected,
	Disconnected,
}

enum Event {
	Dial,
	IncomingCall,
	Answer,
	Reject,
	HangUp,
	Reset,
}

#[derive(Debug, Default)]
struct Shared {
	calls_made: u32,
}

#[derive(Debug)]
enum Error {
	InvalidTransition,
}

#[derive(Debug, Default)]
struct CallEngine;

impl Engine for CallEngine {
	type Error = Error;
	type Event = Event;
	type Shared = Shared;
	type State = State;

	fn next(
		state: &Self::State,
		event: Self::Event,
		shared: &mut Self::Shared,
	) -> Flow<Self::State, Self::Error, Self::Event> {
		use Event::*;
		use Flow::*;
		use State::*;

		let flow = match (state, event) {
			(Idle, Dial) => Transition(Dialing),
			(Dialing, Reject) => Transition(Idle),
			(Dialing, Answer) => Transition(Connected),
			(Connected, HangUp) => Transition(Disconnected),
			(Disconnected, Reset) => Transition(Idle),
			(Idle, IncomingCall) => Transition(Ringing),
			(Ringing, Reject) => Transition(Idle),
			(Ringing, Answer) => Transition(Connected),
			// With `from_residual` feature enabled you can do `Err(InvalidTransition)?`
			_ => Failure(Error::InvalidTransition),
		};

		if let Transition(Connected) = flow {
			shared.calls_made += 1;
		}

		flow
	}
}

type CallMachine = Machine<CallEngine>;

fn main() {
	let machine = CallMachine::default();

	machine.fire(Event::Dial).unwrap();
	machine.fire(Event::Answer).unwrap();
	machine.fire(Event::HangUp).unwrap();

	machine.fire(Event::Reset).unwrap();
	machine.fire(Event::IncomingCall).unwrap();
	machine.fire(Event::Answer).unwrap();
	machine.fire(Event::HangUp).unwrap();

	let destructed = machine.destruct();
	assert_eq!(destructed.state, State::Disconnected);
	assert_eq!(destructed.shared.calls_made, 2);
}
```
