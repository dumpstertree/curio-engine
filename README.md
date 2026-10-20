# 🔮 Curio Engine 

**Curio Engine** is a 3D data driven ECS–hybrid game engine written in **Rust**, designed for building **small, multiplayer-first games** with a strong emphasis on *usability*, *runtime flexibility*, and *modular systems*.

---

## Philosophy

Rather than chasing maximum theoretical performance, Curio Engine prioritizes **approachability and clarity**, even when that means making deliberate trade-offs. The goal is to make engine behavior understandable, quick to implement, and adaptable.

- **Usability first**  
  APIs are designed to be readable and explicit. Engine behavior should be easy to reason about, inspect, and modify.

- **Fully runtime-driven**  
  Nothing is precomputed or baked ahead of time. Curio is optimized for **procedural generation** and dynamic worlds.

- **Modular systems**  
  Most engine systems can be swapped out or replaced with custom implementations to fit your specific needs.
---

## Backing Tech
- **ECS:**         `hecs`
- **Rendering:**   `wgpu`
- **Audio**        `tbd`
- **Animations**   `Spine2D`
---

## Getting Started

A Curio game is built as a `cdylib` and loaded by the engine host through an exported `curio_init` symbol. The `#[curio_entry]` macro generates that symbol for you.

### Creating a Curio

Curios are the application you are building. You take a lame Curio, imbue it with logic and put it on display.

```rust
use curio_core::{Curio, Formation, Identity, InputMapping, Version};
use curio_entry::curio_entry;

#[curio_entry]
pub fn curio() -> Curio {
    // register everything listed in asset.manifest with the asset database.
    // `generated_assets` is emitted by your build script (see "Loading Assets" below).
    Assets::append_listing_group(generated_assets());

    Curio::create()
        // who are we?
        .identity(Identity::new("My Curio", "", Version::new(0, 1, 0)))
        // system components. each is a plugin: time, graphics, input, networking, gameplay, ...
        .plugin(SystemComponentDefaultTime::new())
        .plugin(SystemComponentDefaultGraphics::new())
        .plugin(SystemComponentDefaultInput::new())
        .plugin(SystemComponentDefaultNetworking::new())
        // gameplay is generic over your game events (stimulants) and ui events
        .plugin(SystemComponentDefaultGameplay::<MyStimulant, MyUIEvent>::new())
        // game modes dictate the number of game instances running, their privilege levels
        // and if/how they render to screen. this one is a built in single player layout.
        .formation(Formation::template_local_single(InputMapping::new(
            // map logical names to physical buttons here, e.g.
            // ("paddle_up".to_string(), ButtonCode::KeyW),
            vec![],
            vec![],
        )))
        .imbue()
}
```

And that's all it takes to initialize a new Curio.

### Creating a Habit

Habits are reoccurring loops that happen every "frame". They are not tied to the existence of any object or lifecycle aside from the start and end of the Curio. Habits are automatically picked up by using the [habit] macro.

```rust
#[habit]
pub struct HabitInstance {}

// Defines the scope of the habit (Prereq for Habit). This controls when the habit should startup and stop.
impl Scope for HabitInstance {

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
impl Habit for HabitInstance {

    // triggered when a habit changes from disabled -> enabled or on the first frame of being enabled if always enabled.
    fn enable(&mut self, ledger: &mut Ledger, context: &mut Context3D, nerve: &mut Nerve) {
      println!("Hello World!");
    }

    // triggered when a habit changes from enabled -> disabled
    fn disable(&mut self, ledger: &mut Ledger, context: &mut Context3D, nerve: &mut Nerve) {
      println!("Goodbye World!");
    }

    // triggered every frame a habit is enabled 
    fn tick(&mut self, ledger: &mut Ledger, context: &mut Context3D, nerve: &mut Nerve) {
      println!("I'm Thriving!");
    }
}
```

### Creating a Form and Facets

Forms represent objects that persist in the world (Context) and Facets are the properties that make up that Form. These Forms and Facets are a usability wrapper for HECS ecs to allow the user easier control of objects.

Forms are created with the context spawner and a fluent builder. Facets are constructed with their own builders (for example `Camera::builder()`, `RendererStatic::builder()`, `RendererText::builder()`).

```rust
#[habit]
pub struct HabitInstance {
  camera: Option<Form>,
}
impl Scope for HabitInstance { /* ...same as above... */ }

impl Habit for HabitInstance {

    fn enable(&mut self, ledger: &mut Ledger, context: &mut Context3D, nerve: &mut Nerve) {
      // create a camera to view the world. position/rotation/scale are local to the parent
      // (rotation is euler degrees). children are added with .child( ... .spawn() ).
      self.camera = Some(
        context
          .spawner("My Camera")
          .position(0.0, 10.0, 0.0)
          .facet(Camera::builder().fov(80.0).build())
          .spawn(),
      );
    }

    fn tick(&mut self, _ledger: &mut Ledger, _context: &mut Context3D, _nerve: &mut Nerve) {
      // edit facets on a held form
      if let Some(camera) = &self.camera {
        camera.edit_facet::<Transform3D>(|t| {
          t.position.y = 10.0;
        });
      }
    }

    fn disable(&mut self, _ledger: &mut Ledger, _context: &mut Context3D, _nerve: &mut Nerve) {
      // destroy our camera
      if let Some(camera) = &self.camera {
        camera.destroy();
      }
    }
}
```

Useful `Form` calls: `edit_facet::<T>(|t| ...)`, `get_facet::<T>()`, `has_facet::<T>()`, `get_child("a/b")`, `children()`, `destroy()`.

### Stimulants and Impulses

Stimulants are events that can be sent through your Curio and received by Impulses. Your stimulants are required as part of the startup of your Curio so all events are firmly typed. Stimulants are sent during a frame and then picked up and acted upon at the beginning of the next frame before anything else. If a Stimulant results in another Stimulant the chain will continue before progressing to the Tick phase.

Lets create the object that will hold all our Stimulants. Enums work great for this.

```rust
#[stimulant]
pub enum MyStimulant {
    #[default]
    Invalid,
    Create,
    Destroy,
}
```

Great, now we know what all the Stimulants can be. We can send some from a Habit through the Nerve.

```rust
impl Habit for HabitInstance {

    fn enable(&mut self, ledger: &mut Ledger, context: &mut Context3D, nerve: &mut Nerve) {
        nerve.enqueue_event(MyStimulant::Create);
    }

    fn disable(&mut self, ledger: &mut Ledger, context: &mut Context3D, nerve: &mut Nerve) {
        nerve.enqueue_event(MyStimulant::Destroy);
    }
}
```

Let's now create the Impulse that will react to the Stimulant. Like Habits, Impulses need a `Scope` and are self-registered by the `#[impulse(...)]` macro.

```rust
#[impulse(MyStimulant)]
pub struct ImpulseInstance {}
impl Scope for ImpulseInstance { /* ...same as above... */ }

impl Impulse<MyStimulant> for ImpulseInstance {

  fn dequeue_event(&mut self, ledger: &mut Ledger, context: &mut Context3D, nerve: &mut Nerve, stimulant: &MyStimulant) {
      match stimulant {
          MyStimulant::Create => println!("Stimulant Received: Alive!"),
          MyStimulant::Destroy => println!("Stimulant Received: Dead!"),
          _ => {}
      }
    }
}
```

### Ledger and Records

Records are objects that represent the current state of the Curio. A Ledger is where all those records are kept. Records can be edited but never added or removed as all available records are created when the Curio is created.

Lets create a record to keep track of if we are "alive" or "dead".

```rust
#[record(name = "AliveOrDead", ownership = RecordScope::Instance)]
pub struct MyRecAliveOrDead {
  pub is_alive: bool,
}
```

Now that we have a record that will be picked up with the curio we can edit it from the Impulse.

```rust
impl Impulse<MyStimulant> for ImpulseInstance {

    fn dequeue_event(&mut self, ledger: &mut Ledger, context: &mut Context3D, nerve: &mut Nerve, stimulant: &MyStimulant) {
      match stimulant {
          MyStimulant::Create => {
            ledger.write::<MyRecAliveOrDead>(|rec| { rec.is_alive = true; });
          }
          MyStimulant::Destroy => {
            ledger.write::<MyRecAliveOrDead>(|rec| { rec.is_alive = false; });
          }
          _ => {}
      }
    }
}
```

If we want we can go a step further and check the value was changed in the Habit.

```rust
impl Habit for HabitInstance {

    /// ...
    fn tick(&mut self, ledger: &mut Ledger, context: &mut Context3D, nerve: &mut Nerve) {

        // ledger.read::<T>() returns a read only Rc<T>
        if ledger.read::<MyRecAliveOrDead>().is_alive {
          println!("I'm Alive");
        }
    }
}
```

Some values are built in and populated by the Curio for use and edit such as `SysRecordTime`, `SysRecordInput`, `SysRecordCamera`, `SysRecordScreen`, etc. A few have convenience wrappers. For example, with `use time::ExtensionsLedger`:

```rust
let dt = ledger.time().scaled_delta_time;
let total = ledger.time().scaled_time;
```

Reading input inside a habit (raw keys):

```rust
use input::SysRecordInput;

let input_state = ledger.read::<SysRecordInput>();
if input_state.raw.get_button(&ButtonCode::KeyW).is_down { /* move up */ }
```

If you registered logical names in your `InputMapping`, read them from `input_state.mapped[seat].get_button_or_default("paddle_up")` instead.

### Loading Assets

Assets are registered with the `AssetDatabase` by uid. The recommended flow is a `asset.manifest` file at your project root that your build script turns into a generated `generated_assets()` function. Each entry has a unique `id` and a `type` of `Embedded` (bytes baked into the binary), `Local` (read from disk at runtime) or `Remote` (downloaded and cached).

```yaml
# asset.manifest
manifest:
  - id: 2665
    type: Embedded
    uri: "assets/mesh/Cube.glb"
  - id: 5997
    type: Embedded
    uri: "assets/texture/skybox.png"
```

At startup append the generated listings and load assets by uid wherever you need them:

```rust
Assets::append_listing_group(generated_assets());

// uids are i16. the returned Arc<T> is cached and shared.
let cube = Assets::load_asset::<ModelAsset>(&2665);
```

```rust
context
    .spawner("mesh")
    .facet(RendererStatic::builder().asset(Some(cube)).build())
    .spawn();
```

Names can be looked up too: `Assets::try_lookup_key_for_name("assets/mesh/Cube.glb") -> Option<i16>`.

The engine also registers built in assets you can rely on, such as `ASSET_UID_FONT_ASSET_DEFAULT` (the default font) and built in textures/shaders.

> Note: `build.rs` will fail the build if an embedded asset is missing or empty, and if any id or uri is duplicated — these would otherwise become confusing runtime errors.

### Compositions (prefabs)

Compositions (`.comp` files) are serialized Form hierarchies. Register them in the manifest like any other asset, load them, and hand them to the spawner. Component `type:` is matched against your facet struct name (case insensitive). Each field string is fed to your facet's `FieldOverride::apply`.

```yaml
# assets/compositions/my_comp.comp
enabled: true
base: ''          # optional: uid of another composition to inherit from
name: My GameObject
components:
  - type: RendererText
    fields:
      - 'contents: Hello'
      - 'font_size: 0.055'
      - 'bounds: (5,5)'
children:
  - enabled: true
    name: Child
    components: []
    children: []
    base: '5722'   # spawn this child as a composition with uid 5722
```

```rust
// spawn the same composition 1000 times side by side
let comp = Assets::load_asset::<Composition>(&7902);
for i in 0..1000 {
    let form = context
        .spawner("instance")
        .composition(Some(comp.clone()))
        .spawn();

    form.edit_facet::<Transform3D>(|t| {
        t.position.x = i as f32;
    });
}
```

### Creating a Facet

Facets are defined with the `#[facet]` macro plus a `FieldOverride` impl so they can be edited from compositions and the editor. Registration is automatic (via `ctor`).

```rust
#[facet]
pub struct Rotate {
    pub degrees_per_second: f32,
}
impl FieldOverride for Rotate {
    fn apply(&mut self, field: &str, val: &str) {
        let val = val.trim();
        match field {
            // if we want to override a field from the composition we do so here
            "degrees_per_second" => self.degrees_per_second = val.parse().unwrap_or_default(),
            _ => {}
        }
    }
    fn get_state(&self) -> Vec<FieldState> {
        vec![FieldState::new("degrees_per_second", self.degrees_per_second)]
    }
}
```

Just having the Rotate Facet doesn't DO anything though. It needs a Habit to edit it. This habit is going to be a bit different. It doesn't need references to any Forms and instead will search and edit all Forms in bulk.

```rust
#[habit]
pub struct RotateHabit {}
impl Scope for RotateHabit { /* ...same as above... */ }

impl Habit for RotateHabit {
    fn tick(&mut self, ledger: &mut Ledger, context: &mut Context3D, nerve: &mut Nerve) {
        let dt = ledger.time().scaled_delta_time;

        // query the context for all matches
        context.edit::<(&mut Transform3D, &Rotate)>(|query| {

            // iterate over each match in the query
            for (_, (t, _r)) in query {

                // rotate the transform
                t.rotation = Quaternion::from_euler(Vector3::new(0.0, 5.0 * dt, 0.0)) * t.rotation;
            }
        });
    }
}
```

Now we should have 1000 composition objects all rotating in place.

### Updating Text At Runtime

`RendererText` is built with the builder pattern like the other renderers, and is rebuilt automatically whenever its contents or layout changes:

```rust
// construct
RendererText::builder()
    .contents("Score")
    .font_size(0.1)
    .bounds(Vector2::new(4.0, 4.0))
    .horizontal_alignment(AligmentHorizontal::Center)
    .vertical_alignment(AligmentVertical::Center)
    .font_asset(Some(Assets::load_asset::<FontAsset>(&ASSET_UID_FONT_ASSET_DEFAULT)))
    .tint(Color::white())
    .build();

// mutate a live facet (marks it dirty so it rebuilds this frame)
form.edit_facet::<RendererText>(|t| {
    t.apply("contents", &score.to_string());
});
```
