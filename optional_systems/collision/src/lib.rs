pub use facet::component_collider::BoxColliderDef;
pub use facet::component_collider::ColliderShape;
pub use facet::component_collider::ColliderSnapshot;
pub use facet::component_collider::MeshColliderDef;
pub use facet::component_collider::SphereColliderDef;
pub use record::sys_record_colliders::SysRecordCollider;
pub use record::sys_record_collision::SysRecordCollision;

pub(crate) mod facet {
    pub(crate) mod collider_common;
    pub(crate) mod component_collider;
    pub(crate) mod collider {
        pub(crate) mod collider_box;
        pub(crate) mod collider_sphere;
    }
}

pub(crate) mod record {
    pub(crate) mod sys_record_colliders;
    pub(crate) mod sys_record_collision;
}
