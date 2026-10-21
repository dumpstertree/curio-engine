use curio_core::{Matrix4x4, Quaternion, Vector3};

/// Unified view over `Transform2D` and `Transform3D` so game and habit code can manipulate either
/// without branching on which concrete transform a form carries.
///
/// A `Transform2D` lives on the z=0 plane: reading its position as a `Vector3` yields z=0, and
/// writing a `Vector3` position drops z.
pub trait TransformCommon {
    /// Local position. For `Transform2D` this is `(x, y, 0)`.
    fn position(&self) -> Vector3;
    /// Sets the local position. `z` is dropped for `Transform2D`.
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
