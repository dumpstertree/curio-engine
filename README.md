# Curio Engine 🔮

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

### Creating a World

```rust
// TODO: simple example showing how to create a world
