// use core::{
//     gameplay::ecs::component::component_collider::{ColliderShape, CollisionSnapshot, SphereColliderDef},
//     random::Random,
// };

// pub struct ComponentColliderSphere {
//     pub diameter: f32,
//     pub guid: i32,
//     pub collisions: Vec<CollisionSnapshot>,
// }

// impl ComponentColliderSphere {
//     pub fn default() -> ComponentColliderSphere {
//         ComponentColliderSphere {
//             diameter: 1.0,
//             guid: Random::range_int(-9999, 9999),
//             collisions: Vec::new(),
//         }
//     }
//     pub fn set_diameter(mut self, diameter: f32) -> ComponentColliderSphere {
//         self.diameter = diameter;
//         self
//     }
//     pub fn get_shape(&self) -> ColliderShape {
//         ColliderShape::Sphere(SphereColliderDef { diameter: self.diameter })
//     }
//     pub fn is_colliding(&self) -> bool {
//         self.collisions.len() > 0
//     }
// }
