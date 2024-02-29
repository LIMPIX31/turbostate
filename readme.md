# Turbostate

###### I just needed a state machine, so I wrote this.

## Build engine by example

### 1. Add imports

```rust
use turbostate::engine;
use turbostate::Engine;
```

### 2. Define possible states, events and errors

* `State` - All possible states. There is only one state at a time
* `Event` - The driving part of the machine. By sending events you can change the state.
* `Error` - Point out anything that could go wrong here. If something bad happens, the error will be raised.

```rust
// Required derive: Clone
// Recommended derive: Default
#[derive(Debug, Clone, Default)]
pub enum State {
	// It is good practice to specify a initial state (#[default])
	#[default]
	Disabled,
	Enabled,
	Broken,
}

#[derive(Debug)]
pub enum Event {
	// This event will toggle the switch.
	Toggle,
	// We can deliberately break the switch
	Break,
}

#[derive(Debug, thiserror::Error)]
#[error("The switch is broken")]
struct BrokenSwitchError;

// turbostate captures the required types by a fixed name
// and there is no way to change this yet.
// State and Event should always be named like this,
// if this is not the case, as for example with an error,
// you should create typealias.
type Error = BrokenSwitchError;
```

### 3. Build engine

`Engine` works like a gearbox, you pull the lever(send an event) and change the state

```rust
use Event::*;
// Use all variants for cleaner code
use State::*;

// You can share any data not depending on the state within the engine
// All derives are optional
#[derive(Debug, Default)]
struct SwitchEngine {
	clicks: u32
}

// Or #[engine(async)] for async branches
#[engine]
impl SwitchEngine {
	// branch attribute accepts familiar match arm
	// This works like:
	// match (state, event) {
	//   (Disabled, Switch) => Transition(Enabled),
	//   ...
	// }
	// Note: The return type is not needed
	#[branch((Disabled, Toggle))]
	fn disabled_switch(&mut self) {
		self.clicks += 1;
		// Transition to a new state
		Enabled
	}

	#[branch((Enabled, Toggle))]
	fn enabled_switch(&mut self) {
		self.clicks += 1;
		Disabled
	}

	#[branch((_, Break))]
	fn any_break(&self) {
		Broken
	}

	#[branch((Broken, _))]
	fn broken_any(&self) {
		Err(BrokenSwitchError)
	}

	// Do nothing, e.g if you trying to enable the switch while it is already enabled
	#[branch((state, _))]
	fn rest(&self, state: State) {
		state
	}
}
```

### 4. You can start the engine manually

```rust
fn main() -> anyhow::Result<()> {
	let mut engine = SwitchEngine::default();
	let mut state = State::default();

	state = engine.next(state, Event::Toggle)?;
	assert!(matches!(state, State::Enabled));
	state = engine.next(state, Event::Toggle)?;
	state = engine.next(state, Event::Toggle)?;
	state = engine.next(state, Event::Break)?;
	assert!(matches!(state, State::Broken));
	// ERROR: The switch is broken!
	state = engine.next(state, Event::Toggle)?;

	Ok(())
}
```

> [!Note]
> `turbostate` leaves the implementation of the machine (i.e. what will advance the state) up to you.
