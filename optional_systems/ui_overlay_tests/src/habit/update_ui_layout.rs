#[cfg(test)]
mod tests {
    use curio_core::{AssetCache, AssetDatabase, AssetPipeline, AxisCode, CurioNetwork, CurioNetworkParticipant, Gpu, Ledger, Logger, Nerve, NetworkModes, Quaternion, Services, Severity, Vector2, Vector3};
    use gameplay::{built_in::facet::transform::transform3d::Transform3D, context_3d::Context3D, form::Form, static_data::global_ecs::get_global_ecs_instances};
    use input::SysRecordInput;
    use std::{collections::HashMap, f32::consts::FRAC_PI_2};
    use ui_overlay::{SysRecordUi, UiKind, UiNode};

    // the ledger logs through the Services singleton; the habit never touches the gpu while the record has a viewport
    fn stand_up_services() {
        static SERVICES: std::sync::LazyLock<bool> = std::sync::LazyLock::new(|| {
            let null_void: *const () = std::ptr::null();
            let services: *mut Services = Box::into_raw(Box::new(Services {
                logger: Box::into_raw(Box::new(Logger::new(Severity::Info))),
                assets: Box::into_raw(Box::new(AssetPipeline::new(AssetCache::new(10), AssetDatabase::new()))),
                gpu: Gpu {
                    device: null_void,
                    queue: null_void,
                    capture_texture: null_void,
                    capture_width: 0,
                    capture_height: 0,
                },
            }));
            Services::set(services);
            true
        });
        assert!(*SERVICES);
    }

    fn placed(form: Form, position: Vector3, degrees: f32, scale: Vector3) -> Form {
        let mut transform = Transform3D::default();
        transform.position = position;
        transform.rotation = Quaternion::from_euler(Vector3::new(0.0, 0.0, degrees));
        transform.scale = scale;
        form.add_facet(transform)
    }

    // runs every registered habit once (the ui layout habit is the one this crate links) with the cursor at `cursor`
    fn tick(context: &mut Context3D, cursor: Vector2) -> Ledger {
        let network = CurioNetwork::new(vec![CurioNetworkParticipant::new(0, NetworkModes::LocalHost)], 0);
        let mut ledger = Ledger::new(network.clone());
        let mut nerve = Nerve::new(network);
        ledger.write::<SysRecordUi>(|screen| {
            screen.viewport = Vector2::new(1280.0, 720.0);
            screen.cursor_space = Vector2::new(1280.0, 720.0);
        });
        let mut axes = HashMap::new();
        axes.insert(AxisCode::Cursor, cursor);
        ledger.write::<SysRecordInput>(|state| state.raw.update(&HashMap::new(), &axes));
        let habits = get_global_ecs_instances();
        assert!(!habits.is_empty(), "no habit registered");
        for mut habit in habits {
            habit.tick(&mut ledger, context, &mut nerve);
        }
        ledger
    }

    fn node(form: &Form) -> UiNode {
        match form.get_facet::<UiNode>() {
            Some(node) => node,
            None => panic!("form lost its UiNode"),
        }
    }

    // a card turned a quarter turn at (200, 100) with a word hung inside it, and a flat backdrop behind
    fn card_scene(context: &mut Context3D) -> (Form, Form, Form) {
        let panel = context
            .spawner("hud")
            .facet(UiNode::builder().min_size(1280.0, 720.0).build())
            .spawn();
        let panel = placed(panel, Vector3::zero(), 0.0, Vector3::one());
        let card = context
            .spawner("card")
            .facet(
                UiNode::builder()
                    .kind(UiKind::Block)
                    .size(100.0, 40.0)
                    .build(),
            )
            .spawn();
        card.set_parent(Some(panel.clone()));
        let card = placed(card, Vector3::new(200.0, 100.0, 2.0), 90.0, Vector3::one());
        let word = context
            .spawner("word")
            .facet(
                UiNode::builder()
                    .kind(UiKind::Text)
                    .size(20.0, 10.0)
                    .text("hi")
                    .build(),
            )
            .spawn();
        word.set_parent(Some(card.clone()));
        let word = placed(word, Vector3::new(80.0, 15.0, 1.0), 0.0, Vector3::one());
        let back = context
            .spawner("back")
            .facet(
                UiNode::builder()
                    .kind(UiKind::Block)
                    .size(1280.0, 720.0)
                    .build(),
            )
            .spawn();
        back.set_parent(Some(panel.clone()));
        let back = placed(back, Vector3::new(0.0, 0.0, -1.0), 0.0, Vector3::one());
        (card, word, back)
    }

    #[test]
    fn rotation_scale_and_depth_come_from_transform3d() {
        stand_up_services();
        let world = std::rc::Rc::new(std::cell::RefCell::new(hecs::World::new()));
        let mut context = Context3D::new(world);
        let (card, word, _) = card_scene(&mut context);
        tick(&mut context, Vector2::new(-10.0, -10.0));

        let card_node = node(&card);
        assert_eq!((card_node.rect.x, card_node.rect.y, card_node.rect.width, card_node.rect.height), (200.0, 100.0, 100.0, 40.0));
        assert!((card_node.rotation - FRAC_PI_2).abs() < 0.001, "card turn {}", card_node.rotation);
        assert_eq!(card_node.depth, 2.0);

        // the word sits 40 px right of the card's centre, so the card's quarter turn carries it 40 px up
        let word_node = node(&word);
        let center = word_node.rect.center();
        assert!((center.x - 250.0).abs() < 0.01 && (center.y - 80.0).abs() < 0.01, "word centre {:?}", center);
        assert!((word_node.rotation - FRAC_PI_2).abs() < 0.001);
        assert_eq!(word_node.depth, 3.0);
    }

    #[test]
    fn hover_lands_on_the_card_where_it_is_drawn() {
        stand_up_services();
        let world = std::rc::Rc::new(std::cell::RefCell::new(hecs::World::new()));
        let mut context = Context3D::new(world);
        let (card, _, _) = card_scene(&mut context);
        // inside the upright box's right end, but outside the turned card (x 230..270, y 70..170)
        tick(&mut context, Vector2::new(295.0, 120.0));
        assert!(!node(&card).hovered);
        // below the upright box, inside the turned card
        tick(&mut context, Vector2::new(250.0, 160.0));
        assert!(node(&card).hovered);
    }

    #[test]
    fn scale_sizes_the_drawn_box_about_its_centre() {
        stand_up_services();
        let world = std::rc::Rc::new(std::cell::RefCell::new(hecs::World::new()));
        let mut context = Context3D::new(world);
        let button = context
            .spawner("button")
            .facet(
                UiNode::builder()
                    .kind(UiKind::Button)
                    .size(100.0, 40.0)
                    .build(),
            )
            .spawn();
        let button = placed(button, Vector3::new(100.0, 100.0, 0.0), 0.0, Vector3::new(2.0, 2.0, 1.0));
        // 20 px left of the upright box: only the doubled box reaches it
        let ledger = tick(&mut context, Vector2::new(80.0, 120.0));
        assert!(node(&button).hovered);
        let calls = ledger.read::<SysRecordUi>().draw_calls.clone();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].scale, Vector2::new(2.0, 2.0));
        let corners = calls[0].corners();
        assert_eq!((corners[0].x, corners[0].y, corners[2].x, corners[2].y), (50.0, 80.0, 250.0, 160.0));
    }

    #[test]
    fn draw_calls_paint_in_depth_order() {
        stand_up_services();
        let world = std::rc::Rc::new(std::cell::RefCell::new(hecs::World::new()));
        let mut context = Context3D::new(world);
        let (card, word, back) = card_scene(&mut context);
        let ledger = tick(&mut context, Vector2::new(-10.0, -10.0));
        let order: Vec<i32> = ledger
            .read::<SysRecordUi>()
            .draw_calls
            .iter()
            .map(|call| call.instance_id)
            .collect();
        assert_eq!(order, vec![back.instance_id(), card.instance_id(), word.instance_id()]);
        let calls = ledger.read::<SysRecordUi>().draw_calls.clone();
        assert!((calls[1].rotation - FRAC_PI_2).abs() < 0.001);
    }

    #[test]
    fn an_image_leaf_hands_its_texture_to_the_draw_call() {
        stand_up_services();
        let world = std::rc::Rc::new(std::cell::RefCell::new(hecs::World::new()));
        let mut context = Context3D::new(world);
        let icon = context
            .spawner("icon")
            .facet(
                UiNode::builder()
                    .kind(UiKind::Image)
                    .size(32.0, 32.0)
                    .image(42)
                    .build(),
            )
            .spawn();
        placed(icon, Vector3::new(10.0, 10.0, 0.0), 0.0, Vector3::one());
        let ledger = tick(&mut context, Vector2::new(-10.0, -10.0));
        let calls = ledger.read::<SysRecordUi>().draw_calls.clone();
        assert_eq!(calls.len(), 1);
        assert_eq!((calls[0].kind, calls[0].image), (UiKind::Image, Some(42)));
    }
}
