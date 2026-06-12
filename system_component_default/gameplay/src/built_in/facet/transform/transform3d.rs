use curio_core::{FieldState, Matrix4x4, Quaternion, Vector3};
use std::collections::{HashMap, VecDeque};

use crate::{
    context_3d::Context3D,
    form::Form,
    traits::{facet_common::FacetCommon, field_override::FieldOverride},
    traits_internal::world_context_common::ContextCommon,
};

#[derive(Clone)]
pub struct Transform3D {
    pub position: Vector3,
    pub rotation: Quaternion,
    pub scale: Vector3,
    pub ws_matrix: Matrix4x4,
    owner: Option<Form>,
}
impl FieldOverride for Transform3D {
    fn apply(&mut self, field: &str, val: &str) {
        match field {
            "position" => self.position = val.parse().unwrap_or_default(),
            "rotation" => self.rotation = Quaternion::from_euler(val.parse().unwrap_or_default()),
            "scale" => self.scale = val.parse().unwrap_or_default(),
            _ => {}
        }
    }
    fn get_state(&self) -> Vec<FieldState> {
        vec![
            FieldState::new("position", self.position), //
            FieldState::new("rotation", self.rotation),
            FieldState::new("scale", self.scale),
        ]
    }
}
impl FacetCommon for Transform3D {
    fn set_ownership(&mut self, owner: Form) {
        self.owner = Some(owner);
    }
    fn form(&self) -> Form {
        self.owner.clone().unwrap()
    }
}
impl Default for Transform3D {
    fn default() -> Transform3D {
        Transform3D {
            position: Vector3::zero(),
            rotation: Quaternion::from_look_rotation(Vector3::forward(), Vector3::up()),
            scale: Vector3::one(),
            ws_matrix: Matrix4x4::default(),
            owner: None,
        }
    }
}
unsafe impl Send for Transform3D {}
unsafe impl Sync for Transform3D {}

impl Transform3D {
    pub fn get_matrix(&self) -> Matrix4x4 {
        Matrix4x4::new(self.position, self.rotation, self.scale)
    }
    pub fn move_towards_position(&mut self, position: Vector3, delta: f32) -> f32 {
        let dist = f32::min((position - self.position).magnitude(), delta);
        if dist == 0.0 {
            return 0.0;
        }
        let dir = (position - self.position).normalize_and_copy();

        self.position = self.position + dir * dist;
        dist
    }
    pub fn rotate_towards_rotation(
        &mut self,
        target: Quaternion,
        delta: Vector3, // max euler step per axis (radians or degrees — your choice)
    ) -> f32 {
        // Delta rotation from current to target
        let delta_rot = target * self.rotation.inverse();

        // Convert delta rotation to Euler angles
        let delta_euler = delta_rot.to_euler();

        // Clamp per-axis rotation
        let clamped_euler = Vector3::new(delta_euler.x.clamp(-delta.x, delta.x), delta_euler.y.clamp(-delta.y, delta.y), delta_euler.z.clamp(-delta.z, delta.z));

        // If no rotation is needed
        if clamped_euler.x == 0.0 && clamped_euler.y == 0.0 && clamped_euler.z == 0.0 {
            return 0.0;
        }

        // Apply incremental rotation
        let step = Quaternion::from_euler(clamped_euler);
        self.rotation = step * self.rotation;

        // Return "distance moved" equivalent
        clamped_euler.magnitude()
    }

    pub fn get_world_matrix(&self, _world: &Context3D) -> &Matrix4x4 {
        &self.ws_matrix
    }
}

pub fn update_transform3d(world: &mut Context3D) {
    //
    let mut output = HashMap::new();
    let mut root_transforms = Vec::new();
    let mut all_local_matrix = HashMap::new();

    // iterate over each transform collecting data
    for transform in world.get::<Transform3D>() {
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
        form.edit_facet::<Transform3D>(|x| {
            x.ws_matrix = *matrix;
        });
    }
}
