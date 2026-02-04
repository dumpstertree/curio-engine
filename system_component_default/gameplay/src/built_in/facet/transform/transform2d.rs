use curio_core::{Matrix4x4, Quaternion, Vector2, Vector3};
use std::{
    cell::RefMut,
    collections::{HashMap, VecDeque},
};

use hecs::World;

use crate::{
    context_3d::Context3D,
    form::Form,
    traits::{facet_common::FacetCommon, field_override::FieldOverride},
    traits_internal::world_context_common::ContextCommon,
};

#[derive(Clone)]
pub struct Transform2D {
    // pub parent: Option<Form>,
    pub position: Vector2,
    pub rotation: Quaternion,
    pub scale: Vector3,
    pub ws_matrix: Matrix4x4,
    pub render_order: i32,
    owner: Option<Form>,
}
impl FacetCommon for Transform2D {
    fn set_ownership(&mut self, owner: Form) {
        self.owner = Some(owner);
    }
    fn form(&self) -> Form {
        self.owner.clone().unwrap()
    }
}
impl FieldOverride for Transform2D {
    fn apply(&mut self, field: &str, val: &str) {
        match field {
            "order" => self.render_order = val.parse().unwrap_or_default(),
            "position" => self.position = val.parse().unwrap_or_default(),
            "rotation" => self.rotation = Quaternion::from_euler(val.parse().unwrap_or_default()),
            // "scale" => self.scale = val.parse().unwrap_or_default(),
            _ => {}
        }
    }
}
impl Default for Transform2D {
    fn default() -> Transform2D {
        Transform2D {
            // parent: None,
            position: Vector2::zero(),
            rotation: Quaternion::identity(),
            scale: Vector3::one(),
            ws_matrix: Matrix4x4::default(),
            render_order: 0,
            owner: None,
        }
    }
}
unsafe impl Send for Transform2D {}
unsafe impl Sync for Transform2D {}

impl Transform2D {
    pub fn get_matrix(&self) -> Matrix4x4 {
        let frustrum_w = 2.1; // these values are made up because the camera is perspective
        let frustrum_h = 1.1;
        Matrix4x4::new(
            Vector3::new(
                //
                remap(self.position.x, 0.0, 1.0, -frustrum_w / 2.0, frustrum_w / 2.0),
                remap(self.position.y, 0.0, 1.0, -frustrum_h / 2.0, frustrum_h / 2.0),
                0.01 * self.render_order as f32,
            ),
            self.rotation,
            self.scale,
        )
    }

    pub fn set_render_order(mut self, order: i32) -> Transform2D {
        self.render_order = order;
        self
    }
    pub fn set_position_01(mut self, position: Vector2) -> Transform2D {
        self.position = position;
        self
    }
    pub fn set_rotation(mut self, rotation: Quaternion) -> Transform2D {
        self.rotation = rotation;
        self
    }
    pub fn set_scale(mut self, scale: Vector3) -> Transform2D {
        self.scale = scale;
        self
    }
    pub fn set_parent(self, _parent: Option<Form>) -> Transform2D {
        // self.parent = parent;
        self
    }
    pub fn update_recursive(&self, parent_matrix: Matrix4x4, output: &mut HashMap<Form, Matrix4x4>) {
        let my_matrix = self.get_matrix();
        let final_matrix = Matrix4x4::multiply(&parent_matrix, &my_matrix);
        output.insert(self.form().clone(), final_matrix);
        // self.ws_matrix = Matrix4x4::multiply(&parent_matrix, &my_matrix);
        // for child_form in self.form().children() {
        //     child_form.edit_facet::<Transform2D>(|x| ){
        //     println!("updating child transform2d");
        //     // child_form.edit_facet::<Transform2D>(|x| {
        //     //     x.update_recursive(self.ws_matrix.clone());
        //     // });
        // }
    }
    pub fn update_matrix_in_heirarchy(&mut self, world: &RefMut<'_, World>) -> Matrix4x4 {
        let my_matrix = self.get_matrix();

        if let Some(parent_entity) = self.form().parent() {
            if let Ok(mut parent_renderer) = world.get::<&mut Transform2D>(parent_entity.entity()) {
                let parent_matrix = parent_renderer.update_matrix_in_heirarchy(world);
                self.ws_matrix = Matrix4x4::multiply(&parent_matrix, &my_matrix);
                return self.ws_matrix.clone();
            }
        }
        self.ws_matrix = my_matrix;
        return self.ws_matrix.clone();
    }
    pub fn get_world_matrix(&self, _world: &Context3D) -> &Matrix4x4 {
        &self.ws_matrix
    }
}
pub fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    (value - from_min) / (from_max - from_min) * (to_max - to_min) + to_min
}

pub fn update_transform2d(world: &mut Context3D) {
    //
    let mut output = HashMap::new();
    let mut root_transforms = Vec::new();
    let mut all_local_matrix = HashMap::new();

    // iterate over each transform collecting data
    for transform in world.get::<Transform2D>() {
        // get values for transform
        let form = transform.form();
        let matrix = transform.get_matrix();

        // if we are at root store it as a starting point
        if form.parent().is_none() {
            root_transforms.push(form.clone());
        }

        // insert into stored data
        all_local_matrix.insert(form, matrix);
    }
    // iterate over each transform in root recursively going deeper
    for form in root_transforms {
        // get matrix for this entry
        let parent_matrix = all_local_matrix[&form];

        // add all children in starting point to queue
        let mut child_queue: VecDeque<(Form, Matrix4x4)> = VecDeque::new();
        for child in form.children() {
            child_queue.push_back((child, parent_matrix));
        }

        // add to ouput
        output.insert(form, parent_matrix);

        // while there are children still in the queue
        while let Some((cur_form, parent_ws)) = child_queue.pop_front() {
            // calculate this matrix
            let matrix_parent = parent_ws;
            let matrix_self = all_local_matrix[&cur_form];
            let ws_matrix = Matrix4x4::multiply(&matrix_parent, &matrix_self);

            // add all children to the queue
            let children = cur_form.children();
            for child in children {
                child_queue.push_back((child, ws_matrix));
            }

            // add this to the output
            output.insert(cur_form, ws_matrix);
        }
    }

    // apply all changes with an edit
    for (form, matrix) in &output {
        form.edit_facet::<Transform2D>(|x| {
            x.ws_matrix = *matrix;
        });
    }
}
