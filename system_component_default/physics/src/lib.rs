use rapier3d::{
    na::Isometry3,
    parry::{query, shape::Cuboid},
};

use curio_core::{
    built_in::record::{sys_record_colliders::SysRecordCollider, sys_record_collision::SysRecordCollision},
    gameplay::ecs::component::component_collider::{ColliderShape, CollisionSnapshot, Contact},
};
use curio_core::{
    collections::{event_queue::EventQueue, game_state::GameState, vector3::Vector3},
    gameplay::ecs::component::component_collider::ColliderSnapshot,
    system::{system_component::SystemComponent, system_components::system_component_physics::SystemComponentPhysics},
};

pub struct SystemComponentDefaultPhysics {
    buffer_collider_box: [(Cuboid, ColliderSnapshot); 1024],
    buffer_collider_box_cnt: usize,
    // buffer_collider_ball: [(Ball, Isometry3<f32>, i32); 1024],
    // buffer_collider_ball_cnt: usize,
}

const DEFAULT_CUBE: Cuboid = Cuboid {
    half_extents: rapier3d::na::Vector3::new(0.0, 0.0, 0.0),
};
// const DEFAULT_ISO: Isometry3<f32> = Isometry3 {
//     rotation: Quater { 0: [f32; 4] },
//     translation: f32::consts::FRAC_PI_2,
// };
// const DEFAULT_SNAP: ColliderSnapshot = ColliderSnapshot {
//     guid: 0,
//     matrix: Matrix4x4{ model:  },
//     shape: ColliderShape::Mesh(MeshColliderDef {}),
// };
impl SystemComponentDefaultPhysics {
    pub fn new() -> Box<SystemComponentDefaultPhysics> {
        Box::new(SystemComponentDefaultPhysics {
            buffer_collider_box: [const { (DEFAULT_CUBE, ColliderSnapshot::default()) }; 1024],
            buffer_collider_box_cnt: 0,
            // buffer_collider_ball: [(Ball::new(0.0), Isometry3::identity(), -1); 1024],
            // buffer_collider_ball_cnt: 0,
        })
    }
}
impl SystemComponentDefaultPhysics {}
impl SystemComponentPhysics for SystemComponentDefaultPhysics {}
impl SystemComponent for SystemComponentDefaultPhysics {
    fn order(&self) -> i32 {
        2000
    }
    fn init(&mut self, _: &mut Vec<GameState>) {}

    fn tick(&mut self, game_state: &mut Vec<GameState>, _: &mut Vec<EventQueue>) {
        for game_state in game_state {
            // reset
            self.buffer_collider_box_cnt = 0;

            //
            let state_collider = game_state.get::<SysRecordCollider>();
            for collider in state_collider.colliders {
                // let isometry = Isometry3::identity();
                // let mut shape: &dyn Shape;
                match &collider.shape {
                    ColliderShape::Box(def) => {
                        let size = def.size / 2.0;
                        // self.buffer_collider_box[self.buffer_collider_box_cnt] = Cuboid::new(Vector3::new(size.x, size.y, size.z));
                        self.buffer_collider_box[self.buffer_collider_box_cnt] = (Cuboid::new(rapier3d::na::Vector3::new(size.x, size.y, size.z)), collider);
                        self.buffer_collider_box_cnt = self.buffer_collider_box_cnt + 1;
                    }
                    ColliderShape::Sphere(_) => {
                        // let size = def.diameter / 2.0;
                        // self.buffer_collider_ball[self.buffer_collider_ball_cnt] = Ball::new(size);
                        // self.buffer_collider_ball_cnt = self.buffer_collider_ball_cnt + 1;
                    }
                    ColliderShape::Mesh(_check_system) => todo!(),
                }
            }

            let mut s = game_state.get::<SysRecordCollision>();
            s.collisions.clear();

            for x in 0..self.buffer_collider_box_cnt {
                let xx = &self.buffer_collider_box[x];
                for y in 0..self.buffer_collider_box_cnt {
                    let yy = &self.buffer_collider_box[y];

                    if xx.1.guid == yy.1.guid {
                        continue;
                    }

                    let p0 = xx.1.matrix.extract_position();
                    let r0 = xx.1.matrix.extract_rotation().to_euler();

                    let p1 = yy.1.matrix.extract_position();
                    let r1 = yy.1.matrix.extract_rotation().to_euler();

                    // println!("big rot :{}", r0);

                    let a = Isometry3::new(rapier3d::na::Vector3::new(p0.x, p0.y, p0.z), rapier3d::na::Vector3::new(r0.x, r0.y, r0.z));
                    let b = &xx.0;
                    let c = Isometry3::new(rapier3d::na::Vector3::new(p1.x, p1.y, p1.z), rapier3d::na::Vector3::new(r1.x, r1.y, r1.z));
                    let d = &yy.0;

                    let intersects = query::intersection_test(&a, b, &c, d);

                    let Ok(intersects) = intersects else {
                        continue;
                    };

                    if !intersects {
                        continue;
                    }

                    let contact = query::contact(&a, b, &c, d, 1.0);
                    let Ok(concat) = contact else {
                        continue;
                    };

                    let Some(contacta) = concat else {
                        continue;
                    };

                    s.collisions.push(CollisionSnapshot {
                        collider_a: xx.1.clone(),
                        collider_b: yy.1.clone(),
                        contact: Contact {
                            point: Vector3::new(contacta.point1.x, contacta.point1.y, contacta.point1.z),
                            normal_a: Vector3::new(contacta.normal1.x, contacta.normal1.y, contacta.normal1.z),
                            normal_b: Vector3::new(contacta.normal2.x, contacta.normal2.y, contacta.normal2.z),
                        },
                    });
                }
            }

            game_state.edit::<SysRecordCollision>(|x| {
                x.collisions.clear();
            });
            game_state.edit::<SysRecordCollider>(|x| {
                x.colliders.clear();
            });
        }
    }
}
