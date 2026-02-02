# 🔮 Curio Engine 

**Curio Engine** is a 3D ECS–hybrid game engine written in **Rust**, designed for building **small, multiplayer-first games** with a strong emphasis on *usability*, *runtime flexibility*, and *transparent systems*.

Rather than chasing maximum theoretical performance, Curio prioritizes **approachability and clarity**, even when that means making deliberate trade-offs. The goal is to make engine behavior understandable, debuggable, and adaptable—especially for experimental and procedural game designs.

---

## Philosophy

Curio is built around a few core ideas:

- **Usability first**  
  APIs are designed to be readable and explicit. Engine behavior should be easy to reason about, inspect, and modify.

- **Fully runtime-driven**  
  Nothing is precomputed or baked ahead of time. Curio is optimized for **procedural generation** and dynamic worlds.

- **Constraint-driven design**  
  The engine intentionally supports a limited, well-defined feature set. These constraints encourage better performance characteristics and more interesting design solutions.

- **Modular systems**  
  Most engine systems can be swapped out or replaced with custom implementations to fit your specific needs.

---

## Engine Overview

- **Architecture:** ECS hybrid
- **Rendering:** `wgpu`
- **ECS:** `hecs`
- **Asset support:**
  - Static **glTF** meshes
  - **Spine2D** animations
- **Runtime model:** Fully dynamic, no pre-baked data
- **Multiplayer model:**  
  Designed to run **multiple game instances simultaneously**, enabling:
  - Seamless local multiplayer
  - Online multiplayer
  - Single-player using the same code paths

---

## Core Concepts

> (Brief explanations live here — keep these short and conceptual)

- **World** – Holds entities, components, and runtime state
- **Systems** – Logic that operates on world data
- **Components** – Data attached to entities
- **States** – Shared, editable runtime data
- **Facets / Prefabs** – Reusable composition patterns

---

## Getting Started

### Creating a Curio

Curios are an application that you are building. You take a lame Curio, imbue it with logic and put it on display.

```rust
    CurioCabinet::display_curio(
        CurioMetadata::new(
            "Volleyball", //
            "icon.png",
            VersionNumber::new(0, 1, 0),
        ),
        || {
            Curio::imbue(
                vec![
                    SystemComponentDefaultTime::new(),
                    SystemComponentDefaultInput::new(),
                    SystemComponentDefaultPhysics::new(),
                    SystemComponentDefaultGraphics::new(),
                    SystemComponentDefaultNetworking::new(),
                    SystemComponentDefaultGameplay::<GameEvents, UIViewTypes>::new(),
                ],
                GameMode::new_local_single(
                    InputMapping::new(
                        vec![
                            (String::from("card_mode"), ButtonCode::ShiftLeft),
                            (String::from("move_forward"), ButtonCode::KeyW),
                            (String::from("move_back"), ButtonCode::KeyS),
                            (String::from("move_left"), ButtonCode::KeyA),
                            (String::from("move_right"), ButtonCode::KeyD),
                            (String::from("turn_end"), ButtonCode::KeyP),
                            (String::from("card_left"), ButtonCode::KeyA),
                            (String::from("card_right"), ButtonCode::KeyD),
                            (String::from("card_submit"), ButtonCode::ArrowUp),
                        ],
                        vec![],
                    )
                ),
            )
        },
        WindowLayout::fullscreen_1080(),
    );
}
```


### Creating a Habit

Habits a reoccuring loop that happen every "frame". They are not tied to the existance of any object or lifecycle aside from the start and end of the Curio. Habits are automatically picked up by using the [habit] macro. 

```rust
#[habit]
pub struct Instance {}

// Defines the scope of the habit (Prereq for Habit). This controls when the habit should startup and stop.
impl Scope for Instance {

    fn is_enabled(&mut self, ledger: &mut Ledger) -> bool {
      // always run
      true 
    }
    fn run_on_instance(&mut self, ledger: &mut Ledger) -> Vec<NetworkModes> {
      // on all instances
      NetworkModes::all()
    }
}

// Defines the logic for the habit
impl Habit for Instance {

    // triggered when a habit changes from disabled -> enabled or on the first frame of being enabled if always enabled.
    fn enable(&mut self, ledger: &mut Ledger, context: &mut Context3D, event_queue: &mut EventQueue) {
      println!("Hello World!");
    }

    // triggered when a habit changes from enabled -> disabled
    fn disable(&mut self, ledger: &mut Ledger, context: &mut Context3D, event_queue: &mut EventQueue) {
      println!("Goodbye World!");
    }

    // triggered every frame a habit is enabled 
    fn tick(&mut self, ledger: &mut Ledger, context: &mut Context3D, event_queue: &mut EventQueue) {
      println!("I'm Thriving!");
    }
}
```
### Creating a Form and Facets

Forms represent objects that persist in the world (Context) and Facets are the properties that make up that Form. These Forms and Facets are a usability wrapper for HECS ecs to allow the user easier control of objects.

```rust
#[habit]
pub struct Instance {
  camera: Form
}
impl Scope for Instance {
    fn is_enabled(&mut self, ledger: &mut Ledger) -> bool {
      true 
    }
    fn run_on_instance(&mut self, ledger: &mut Ledger) -> Vec<NetworkModes> {
      NetworkModes::all()
    }
}
impl Habit for Instance {
    fn enable(&mut self, ledger: &mut Ledger, context: &mut Context3D, event_queue: &mut EventQueue) {
      // create a camera to view the world
      self.camera = context.spawn( "My Camera", Transform3D::default()).add_facet_default::<Camera>();
    }
    fn disable(&mut self, ledger: &mut Ledger, context: &mut Context3D, event_queue: &mut EventQueue) {
      // destroy out camera
      self.camera.destroy();
    }
    fn tick (&mut self, ledger: &mut Ledger, context: &mut Context3D, event_queue: &mut EventQueue) {
      // edit the camera position to move up and down
      self.camera.edit_facet::<Transform3D>( |t| {
        // update position
      });
    }
}
```

### Creating Custom Facets 


### Stimulants and Impulses

Stimulants are events that can be sent through your Curio and recieved by Impulses. Your stimulants are required as part of the startup of your Curio so all events are firmly typed. Stimulants are sent during a frame and then picked up and acted upon at the beggining of the next frame before anything else. If a Stimulant results in another Stimulant the chain will continue before progressing to the Tick phase.


Lets create the object that will hold all our Stimulants. Enums work great for this.

```rust
// all out stimulants in the Curio
#[stimulant]
pub enum MyStimulant {
    #[default]
    Invalid,
    Create,
    Destroy
}

```

Great now we know what all the Stimulants can be we can send some from a Habit.

```rust
// ... 
impl Habit for HabitInstance {
    fn enable(&mut self, ledger: &mut Ledger, context: &mut Context3D, event_queue: &mut EventQueue) {
        event_queue.enqueue_event( MyStimulant::Create );
    }
    fn disable(&mut self, ledger: &mut Ledger, context: &mut Context3D, event_queue: &mut EventQueue) {
        event_queue.enqueue_event( MyStimulant::Destroy );
    }
}
```

Let's now create the Impluse that will react to the Stimulant

```rust
// reciever the stimulant 
#[impulse(MyStimulant)]
pub struct ImpulseInstance {}
impl Impulse<MyStimulant> for ImpulseInstance {
    fn dequeue_event(&mut self, ledger: &mut Ledger, context: &mut Context3D, event_queue: &mut EventQueue, event: &GameEvents) {
      match event {
          MyStimulant::Create => println1( "Stimulant Recieved: Alive!");
          MyStimulant::Destroy => println1( "Stimulant Recieved: Dead!");
      }
    }
}
```

### Ledger and Records

Records are objects that represent the current state of the Curio. A Ledger is where all those records are kept. Records can be edited but never added or removed as all available records are created when the Curio is created.

Lets create a record to keep track of if we are "alive" or "dead"

```rust

#[record]
pub struct MyRecAliveOrDead {
  is_alive: bool
}
impl Record for MyRecAliveOrDead {
    fn ownership() -> StateOwnerships {
        StateOwnerships::Instance
    }
}

```
Now that we have a record that will be picked up with the curio we can edit the Stimulant to edit it. 

```rust

impl Impulse<MyStimulant> for ImpulseInstance {
    fn dequeue_event(&mut self, ledger: &mut Ledger, context: &mut Context3D, event_queue: &mut EventQueue, event: &GameEvents) {
      match event {
          MyStimulant::Create => {
            ledger.edit::<MyRecAliveOrDead>( |rec| { rec.is_alive = true });
          }
          MyStimulant::Destroy => {
            ledger.edit::<MyRecAliveOrDead>( |rec| { rec.is_alive = false });
          }
      }
    }
}

```
If we want we can go a step further and check the value was changed in the Habit.

```rust
// ... 
impl Habit for HabitInstance {

    /// ...
    fn tick(&mut self, ledger: &mut Ledger, context: &mut Context3D, event_queue: &mut EventQueue) {
         if ledger.get::<MyRecAliveOrDead>().is_alive {
          println!("I'm Alive");
        }
    }
}
```

Some values are built in and populated by the Curio for use and edit such as SysRecordTime, SysRecordInput, SysRecordCamera, SysRecordScreen, etc. Many of these have convience wrappers from the Ledger. For example...

```rust

ledger.time().delta_time
ledger.camera().fov
ledger.screen().width

```
