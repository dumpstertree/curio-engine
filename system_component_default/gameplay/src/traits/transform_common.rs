use curio_core::{Matrix4x4, Quaternion, Vector3};

/// A transform's position, rotation and scale behind one interface, so habit code need not name the facet.
pub trait TransformCommon {
    fn position(&self) -> Vector3;
    fn set_position(&mut self, position: Vector3);
    fn rotation(&self) -> Quaternion;
    fn set_rotation(&mut self, rotation: Quaternion);
    /// Convenience for `set_rotation(Quaternion::from_euler(degrees))`.
    fn set_rotation_euler(&mut self, degrees: Vector3) {
        self.set_rotation(Quaternion::from_euler(degrees));
    }
    fn scale(&self) -> Vector3;
    fn set_scale(&mut self, scale: Vector3);
    /// The local TRS matrix for this transform.
    fn local_matrix(&self) -> Matrix4x4;
}
