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

Habits a reoccuring loop that happen every "frame". They are not tied to the existance of any object or lifecycle aside from the start of the Curio.

```rust
#[habit]
pub struct Instance {}

// Defines the scope of the habit (Prereq for Habit). This controls when the habit should startup and stop.
impl Scope for Instance {

    //
    fn is_enabled(&mut self, ledger: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, ledger: &mut Ledger) -> Vec<NetworkModes> {
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

