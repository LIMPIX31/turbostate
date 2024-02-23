# Turbostate

###### I just needed a state machine, so I wrote this.

## Features

| feature         | description                                                          |
|-----------------|----------------------------------------------------------------------|
| `from_residual` | Short-circuit error conversion from `Result::Err` to `Flow::Failure` |

## By example

### 1. Add imports
```rust
use turbostate::engine;
use turbostate::Machine;
// Use all variants for cleaner code
use turbostate::Flow::*;
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
// You can share any data not depending on the state within the engine
// All derives are optional
#[derive(Debug, Default)]
struct Engine {
  clicks: u32
}

// Use all variants for cleaner code
use State::*;
use Event::*;

#[engine]
impl Engine {
  // branch attribute accepts familiar match arm
  // This works like:
  // match (state, event) {
  //   (Disabled, Switch) => Transition(Enabled),
  //   ...
  // }
  // Note: The return type is not needed
  #[branch((Disabled, Switch))]
  async fn disabled_switch(&mut self) {
    self.clicks += 1;
    // Transition to a new state
    Transition(Enabled)
  }

  #[branch((Enabled, Switch))]
  async fn enabled_switch(&self) {
    self.clicks += 1;
    Transition(Disabled)
  }

  #[branch((_, Break))]
  async fn any_break(&self) {
    Transition(Break)
  }

  #[branch((Broken, _))]
  async fn broken_any(&self) {
    Failure(BrokenSwitchError)
  }

  // Do nothing, e.g if you trying to enable the switch while it is already enabled
  #[branch(_)]
  async fn rest(&self) {
    Pass
  }
}
```

### 4. Run machine by firing events

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Since `Engine` and `State` implement Default you can use `default` method from `Default` crate
  let mut machine = Machine::<Engine>::default();

  machine.fire(Event::Enable).await?;
  machine.fire(Event::Disable).await?;
  machine.fire(Event::Enable).await?;
  machine.fire(Event::Break).await?;
  // The switch is broken!
  machine.fire(Event::Enable).await?;

  Ok(())
}
```
