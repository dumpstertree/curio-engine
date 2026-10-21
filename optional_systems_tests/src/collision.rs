#[cfg(test)]
mod tests {
    use collision::{Collider, ColliderShapeDef, ColliderSnapshot, CollisionPair, Contact, SysRecordCollider, SysRecordCollision};
    use curio_core::{FieldState, Matrix4x4, RecordOverride, Vector3};
    use gameplay::traits::field_override::FieldOverride;

    fn state<'a>(fields: &'a [FieldState], name: &str) -> &'a serde_json::Value {
        fields
            .iter()
            .find(|f| f.field_name == name)
            .map(|f| &f.data)
            .unwrap_or_else(|| panic!("field '{}' missing from get_state", name))
    }

    fn snapshot(guid: i32, sensor: bool, group: u32, mask: u32) -> ColliderSnapshot {
        ColliderSnapshot {
            guid,
            matrix: Matrix4x4::default(),
            shape: ColliderShapeDef::default(),
            sensor,
            filter_group: group,
            filter_mask: mask,
        }
    }

    // ---------- defaults ----------

    #[test]
    fn collider_defaults() {
        let collider = Collider::default();
        assert_eq!(collider.shape, ColliderShapeDef::Box { size: Vector3::one() });
        assert!(!collider.sensor);
        assert_eq!(collider.filter_group, 0);
        assert_eq!(collider.filter_mask, 0);
    }

    // ---------- builder ----------

    #[test]
    fn builder_sets_every_field() {
        let collider = Collider::builder()
            .capsule_shape(0.4, 2.0)
            .sensor(true)
            .filter_group(2)
            .filter_mask(5)
            .build();
        assert_eq!(collider.shape, ColliderShapeDef::Capsule { radius: 0.4, height: 2.0 });
        assert!(collider.sensor);
        assert_eq!(collider.filter_group, 2);
        assert_eq!(collider.filter_mask, 5);
    }

    #[test]
    fn builder_shape_helpers_override_last_shape() {
        let collider = Collider::builder().box_shape(Vector3::new(1.0, 2.0, 3.0)).sphere_shape(4.0).build();
        assert_eq!(collider.shape, ColliderShapeDef::Sphere { diameter: 4.0 });
    }

    // ---------- filters ----------

    #[test]
    fn zero_mask_collides_with_everything() {
        let a = snapshot(1, false, 0, 0);
        let b = snapshot(2, false, 8, 0); // mask 0 => this collider accepts anything
        assert!(a.filter_allows(&b));
        assert!(b.filter_allows(&a));
    }

    #[test]
    fn matching_group_and_mask_allows() {
        let a = snapshot(1, false, 1, 2); // group 1, wants 2
        let b = snapshot(2, false, 2, 1); // group 2, wants 1
        assert!(a.filter_allows(&b));
        assert!(b.filter_allows(&a));
    }

    #[test]
    fn one_sided_mask_blocks() {
        // a wants 2 but b (group 4) does not want a's group
        let a = snapshot(1, false, 1, 2);
        let b = snapshot(2, false, 4, 0);
        assert!(!a.filter_allows(&b));
        assert!(!b.filter_allows(&a));
    }

    // ---------- shape encode / decode ----------

    #[test]
    fn shape_round_trips_through_encode_decode() {
        let shapes = vec![
            ColliderShapeDef::Box { size: Vector3::new(1.0, 2.0, 3.0) },
            ColliderShapeDef::Sphere { diameter: 2.5 },
            ColliderShapeDef::Capsule { radius: 0.5, height: 1.75 },
        ];
        for shape in shapes {
            let decoded = ColliderShapeDef::decode(&shape.encode());
            assert_eq!(decoded, Some(shape.clone()), "failed to round trip {:?}", shape);
        }
    }

    #[test]
    fn decode_rejects_garbage() {
        assert_eq!(ColliderShapeDef::decode("triangle 1,2,3"), None);
        assert_eq!(ColliderShapeDef::decode(""), None);
        assert_eq!(ColliderShapeDef::decode("box"), None);
        assert_eq!(ColliderShapeDef::decode("box (1,2)"), None);
        assert_eq!(ColliderShapeDef::decode("sphere nope"), None);
    }

    // ---------- facet apply / get_state ----------

    #[test]
    fn apply_shape_updates() {
        let mut collider = Collider::default();
        collider.apply("shape", "sphere 3.0");
        assert_eq!(collider.shape, ColliderShapeDef::Sphere { diameter: 3.0 });
    }

    #[test]
    fn apply_bad_shape_keeps_current() {
        let mut collider = Collider::builder().sphere_shape(3.0).build();
        collider.apply("shape", "nonsense");
        assert_eq!(collider.shape, ColliderShapeDef::Sphere { diameter: 3.0 });
    }

    #[test]
    fn apply_sensor_group_mask() {
        let mut collider = Collider::default();
        collider.apply("sensor", "true");
        collider.apply("group", "4");
        collider.apply("mask", "1");
        assert!(collider.sensor);
        assert_eq!(collider.filter_group, 4);
        assert_eq!(collider.filter_mask, 1);
    }

    #[test]
    fn apply_bad_values_keep_current() {
        let mut collider = Collider::builder().sensor(true).filter_group(3).filter_mask(7).build();
        collider.apply("sensor", "maybe");
        collider.apply("group", "abc");
        collider.apply("mask", "xyz");
        assert!(collider.sensor);
        assert_eq!(collider.filter_group, 3);
        assert_eq!(collider.filter_mask, 7);
    }

    #[test]
    fn get_state_exposes_all_fields() {
        let collider = Collider::builder().sphere_shape(2.0).sensor(true).filter_group(2).filter_mask(9).build();
        let fields = collider.get_state();
        assert_eq!(state(&fields, "shape").as_str(), Some("sphere 2"));
        assert_eq!(state(&fields, "sensor").as_bool(), Some(true));
        assert_eq!(state(&fields, "group").as_u64(), Some(2));
        assert_eq!(state(&fields, "mask").as_u64(), Some(9));
    }

    // ---------- pair helpers ----------

    #[test]
    fn pair_helpers_find_the_other_side() {
        let pair = CollisionPair {
            collider_a: snapshot(5, false, 0, 0),
            collider_b: snapshot(9, false, 0, 0),
            contact: Some(Contact {
                point: Vector3::zero(),
                normal_a: Vector3::new(1.0, 0.0, 0.0),
                normal_b: Vector3::new(-1.0, 0.0, 0.0),
            }),
        };
        assert!(pair.involves(5));
        assert!(pair.involves(9));
        assert!(!pair.involves(1));
        assert_eq!(pair.other_guid(5), Some(9));
        assert_eq!(pair.other_guid(9), Some(5));
        assert_eq!(pair.other_guid(1), None);
    }

    // ---------- records ----------

    #[test]
    fn collider_record_starts_empty() {
        let record = SysRecordCollider::default();
        assert!(record.colliders.is_empty());
    }

    #[test]
    fn collision_record_starts_empty() {
        let record = SysRecordCollision::default();
        assert!(record.overlaps.is_empty());
        assert!(record.started.is_empty());
        assert!(record.ended.is_empty());
    }

    #[test]
    fn collision_record_state_shows_counts() {
        let record = SysRecordCollision::default();
        let fields = record.get_state();
        assert_eq!(state(&fields, "overlaps").as_str(), Some("0"));
        assert_eq!(state(&fields, "started").as_str(), Some("0"));
        assert_eq!(state(&fields, "ended").as_str(), Some("0"));
    }
}
