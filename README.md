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

### Creating a Curio

Curios are an application that you are building. You take a lame Curio, imbue it with logic and put it on display.

```rust
    CurioCabinet::display_curio(
        CurioMetadata::new(
            "My Curio", "icon.png", VersionNumber::new(0, 1, 0),
        ),
        || {
            Curio::imbue(
                // here we can edit this to override with our own custom systems. We will talk about MyStimulant later.
                default_systems::<MyStimulant>(),

                // game modes dictate the number of game instances running, their privilege levels and if/how they render to screen.
                // for now we are using a built in single player version
                GameMode::new_local_single(

                  // here we can override this to create our own custom input mapping
                  default_input(),
                ),
            )
        },

        // this is the default graphics settings the curio will launch with
        WindowLayout::fullscreen_1080(),

        // this is where any assets we want to include are stored.
        // we will go into more detail on this later.
        AssetDatabase::new(),
    );
}
```

And thats all it takes to initialize a new Curio.

### Creating a Habit

Habits a reoccuring loop that happen every "frame". They are not tied to the existance of any object or lifecycle aside from the start and end of the Curio. Habits are automatically picked up by using the [habit] macro. 

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
impl Habit for Instance {

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

```rust
#[habit]
pub struct HabitInstance {
  camera: Form
}
impl Scope for HabitInstance {

   fn is_enabled(&mut self, ledger: &mut Ledger) -> bool {
      true 
    }

    fn run_on_instance(&mut self, ledger: &mut Ledger) -> Vec<NetworkModes> {
      NetworkModes::all()
    }
}
impl Habit for HabitInstance {

    fn enable(&mut self, ledger: &mut Ledger, context: &mut Context3D, nerve: &mut Nerve) {
      // create a camera to view the world
      self.camera = context.spawn( "My Camera", Transform3D::default()).add_facet_default::<Camera>();

      // update camera position
      self.camera.edit_facet::<Transform3D>( |t| {
        t.position = Vector3::new( 0.0, 10.0, 0.0);
      });
    }

    fn disable(&mut self, ledger: &mut Ledger, context: &mut Context3D, nerve: &mut Nerve) {
      // destroy out camera
      self.camera.destroy();
    }
}
```

### Stimulants and Impulses

Stimulants are events that can be sent through your Curio and recieved by Impulses. Your stimulants are required as part of the startup of your Curio so all events are firmly typed. Stimulants are sent during a frame and then picked up and acted upon at the beggining of the next frame before anything else. If a Stimulant results in another Stimulant the chain will continue before progressing to the Tick phase.


Lets create the object that will hold all our Stimulants. Enums work great for this.

```rust
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
... 
impl Habit for HabitInstance {

    fn enable(&mut self, ledger: &mut Ledger, context: &mut Context3D, nerve: &mut Nerve) {
        nerve.stimulate( MyStimulant::Create );
    }

    fn disable(&mut self, ledger: &mut Ledger, context: &mut Context3D, nerve: &mut Nerve) {
        nerve.stimulate( MyStimulant::Destroy );
    }
}
```

Let's now create the Impluse that will react to the Stimulant

```rust
#[impulse(MyStimulant)]
pub struct ImpulseInstance {}
impl Impulse<MyStimulant> for ImpulseInstance {

  fn stimulate(&mut self, ledger: &mut Ledger, context: &mut Context3D, nerve: &mut Nerve, stimulant: &MyStimulant) {
      
      match stimulant {
          MyStimulant::Create => println!( "Stimulant Recieved: Alive!");
          MyStimulant::Destroy => println!( "Stimulant Recieved: Dead!");
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
  pub is_alive: bool
}
impl Record for MyRecAliveOrDead {
    fn ownership() -> StateOwnerships {
        RecordOwnerships::Instance
    }
}

```
Now that we have a record that will be picked up with the curio we can edit the Stimulant to edit it. 

```rust

impl Impulse<MyStimulant> for ImpulseInstance 
    
    fn stimulate(&mut self, ledger: &mut Ledger, context: &mut Context3D, nerve: &mut Nerve, stimulant: &MyStimulant) {
      match stimulant {
          MyStimulant::Create => {
            ledger.edit::<MyRecAliveOrDead>( |rec| { rec.is_alive = true; });
          }
          MyStimulant::Destroy => {
            ledger.edit::<MyRecAliveOrDead>( |rec| { rec.is_alive = false; });
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
    fn tick(&mut self, ledger: &mut Ledger, context: &mut Context3D, nerve: &mut Nerve) {

        // When using ledger.get<T> a readonly value of Arc<T> is returned.
        if ledger.get::<MyRecAliveOrDead>().is_alive {
          println!("I'm Alive");
        }
    }
}
```

Some values are built in and populated by the Curio for use and edit such as SysRecordTime, SysRecordInput, SysRecordCamera, SysRecordScreen, etc. Many of these have convience wrappers from the Ledger. For example...

```rust

let x = ledger.time().delta_time;
or 
let x = ledger.camera().fov;
or
let x = ledger.screen().width;
```

### Loading Assets and Prefabs

Like Stimulants assets and their locations are defined when the Curio is constructed. Assets can be Local or Remote. Local assets are pulled from the local asset folder/$asset_type and remote assets are stored in a cache. When assets are loaded they are stored in memory and added to an asset cache until a memory quota is hit and less used assets are released.

Lets add some entries to the AssetDatabase that we created at the start.

```rust
    CurioCabinet::display_curio(

        //...
        AssetDatabase::new_from_explicit(vec![
          (001, "my_prefab", AssetDatabaseListing::Local(String::from("prefab/my_prefab.prefab")))
        ])
    );
}
```

Lets now look at that the contents of prefab/myprefab.prefab.

```yaml
name: "My Prefab"
facets:
  - type: "transform3d"
    fields:
      - "position: (0.0,0.0)"
      - "scale: (1,1,1)"
  - type: "renderertext"
    fields:
      - "contents:"My Prefab"
      - "font_size:0.055"
      - "bounds:(5,5)"
  - type: "rotate"
    fields: []
children: []
```

Now that we have an asset we can load them into the context. Lets spawn this as well at the start of the Habit. This prefab though is missing a Facet in our project, "Rotate". Lets define that now

```rust
#[facet]
pub struct Rotate {}
impl FieldOverride for Rotate {
    fn apply(&mut self, _field: &str, _val: &str) {

      // if we want to override a field from the prefab we do so here
    }
}

```
Now that we have all the dependencies can spawn our prefab.

```rust
//... 
impl Habit for HabitInstance {

    //...
    fn enable(&mut self, ledger: &mut Ledger, context: &mut Context3D, nerve: &mut Nerve) {
      nerve.stimulate( MyStimulant::Create );

      // lets create a bunch of these
      for i in 0..1000 {

            // spawn the prefab using its id 
            let form = context.spawn_prefab( &AssetLoader::load::<Prefab>(001) );

            // we'll edit each transform to all sit side by side.
            form.edit::<Transform3D>( |t| {
              t.position = Vector3::new( i , 0.0, 0.0 );
            });
        }
    }
}
```
Just having the Rotate Facet doesn't DO anything though. It needs a Habit to edit it. This habbit is going to be a bit different. It doesn't need references to any Forms and instead will search and edit all Forms in bulk.

```rust
#[habit]
pub struct RotateHabit {}
impl Scope for RotateHabit {
    fn is_enabled(&mut self, ledger: &mut Ledger) -> bool {
      true 
    }
    fn run_on_instance(&mut self, ledger: &mut Ledger) -> Vec<NetworkModes> {
      NetworkModes::all()
    }
}
impl Habit for RotateHabit {
    fn tick(&mut self, ledger: &mut Ledger, context: &mut Context3D, nerve: &mut Nerve) {

        // query the context for all matches
        context.edit::<(Transform3D, Rotate)>( |query| {

          // iterate over each match in the query
          for (_, (t,r) in query {

            // rotate the transform
            t.rotation *= Quaternion::from_euler( 0.0, 5.0 * ledger.time().delta_time, 0.0 );
        }
    }
}
    
```

Now we should have 1000 prefab objects all rotating in place.


