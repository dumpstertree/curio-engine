use curio_core::{Quaternion, Vector3};

/// Where a unit quad on a `Transform3D` must sit to cover a ui rect; from `UiScreen::place_in_world`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct UiWorldPlacement {
    pub position: Vector3,
    pub rotation: Quaternion,
    pub scale: Vector3,
}
