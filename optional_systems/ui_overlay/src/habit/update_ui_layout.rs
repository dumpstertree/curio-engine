use curio_core::{AxisCode, ButtonCode, Color, Ledger, Nerve, NetworkModes, Quaternion, Services, Severity, Vector2, Vector3};
use gameplay::traits::facet_common::FacetCommon;
use gameplay::{
    built_in::facet::transform::transform3d::Transform3D,
    context_3d::Context3D,
    form::Form,
    traits::{habit::Habit, scope::Scope},
    traits_internal::world_context_common::ContextCommon,
};
use habit::habit;
use input::SysRecordInput;
use std::collections::{HashMap, HashSet};
use taffy::{
    geometry::{Rect, Size},
    style::{AvailableSpace, Dimension, Display, FlexDirection, LengthPercentage, LengthPercentageAuto, Position, Style},
    style_helpers::{auto, length},
    tree::{NodeId, TaffyTree},
};

use crate::{SysRecordUi, UiDrawCall, UiKind, UiNode, UiRect};

// one UI node's bookkeeping for this frame, keyed by the instance id of
// the form that owns it
struct UiEntry {
    pub form: Form,
    pub node: UiNode,
    // this node's top form inside its UI parent (None when it is a root).
    // instance ids can be any i32, negative included, so "no parent" is
    // never spelled as an id
    pub top_id: Option<i32>,
    // the nearest UI ancestor above this node (None when it is a root)
    pub parent_ui: Option<i32>,
    // how many UI ancestors sit above it
    pub depth: u32,
    // its Transform3D offset, accumulated up to the UI parent: x/y in pixels, z its layer
    pub offset_x: f32,
    pub offset_y: f32,
    pub offset_z: f32,
    // its own Transform3D turn (radians, counter-clockwise) and x/y scale
    pub rotation: f32,
    pub scale: Vector2,
}

impl UiEntry {
    pub fn new(node: UiNode) -> UiEntry {
        UiEntry {
            form: node.form(),
            node,
            top_id: None,
            parent_ui: None,
            depth: 0,
            offset_x: 0.0,
            offset_y: 0.0,
            offset_z: 0.0,
            rotation: 0.0,
            scale: Vector2::one(),
        }
    }
}

// authored numbers go straight into taffy, so clamp the nonsense away
fn finite(value: f32) -> f32 {
    if value.is_nan() || value.is_infinite() { 0.0 } else { value }
}
fn safe_size(value: f32) -> f32 {
    let value = finite(value);
    if value < 0.0 {
        0.0
    } else if value > 1000000.0 {
        1000000.0
    } else {
        value
    }
}

// the screen is flat, so only the turn about z counts: where the x axis ends up
fn planar_angle(rotation: Quaternion) -> f32 {
    let axis = rotation * Vector3::new(1.0, 0.0, 0.0);
    finite(axis.y.atan2(axis.x))
}

// hover/click feedback scales the rgb channels and keeps the alpha. curio_core
// only offers Color * f32, which scales alpha too — an alpha over 1 lets the
// blend darken whatever sits behind the box
fn clamp_01(value: f32) -> f32 {
    if value < 0.0 {
        0.0
    } else if value > 1.0 {
        1.0
    } else {
        value
    }
}
fn brighten(color: Color, amount: f32) -> Color {
    Color::new_01(clamp_01(color.as_r_01() * amount), clamp_01(color.as_g_01() * amount), clamp_01(color.as_b_01() * amount), color.as_a_01())
}
// Color's own Default is fully clear, so an authored box that never got a
// colour would paint nothing — white is what the author meant
fn visible_color(color: Color) -> Color {
    if color == Color::clear() { Color::white() } else { color }
}

// the taffy style for one node. Panels flex their children; everything else
// is an absolutely-positioned leaf hung off its UI parent
fn ui_style(node: &UiNode, offset_x: f32, offset_y: f32) -> Style {
    let zero_lpa: LengthPercentageAuto = length(0.0);
    let size_auto: Size<Dimension> = Size { width: auto(), height: auto() };
    let min_auto: Size<LengthPercentageAuto> = Size { width: auto(), height: auto() };
    let gap_none: Size<LengthPercentage> = Size { width: length(0.0), height: length(0.0) };
    let inset_none: Rect<LengthPercentageAuto> = Rect {
        left: zero_lpa,
        right: zero_lpa,
        top: zero_lpa,
        bottom: zero_lpa,
    };
    let mut style = Style {
        display: Display::Flex,
        size: size_auto,
        min_size: min_auto,
        gap: gap_none,
        position: Position::Relative,
        inset: inset_none,
        flex_grow: 0.0,
        flex_direction: FlexDirection::Row,
        ..Default::default()
    };
    if node.kind != UiKind::Panel {
        // leaves paint a box and hang at their Transform3D offset inside
        // the nearest UI ancestor
        style.display = Display::Block;
        style.position = Position::Absolute;
        style.inset.left = length(finite(offset_x));
        style.inset.top = length(finite(offset_y));
    } else if node.absolute {
        // an authored free-form frame: hang at its own Transform3D offset
        // inside the nearest UI ancestor instead of floating in flex flow;
        // it still flexes its own children
        style.position = Position::Absolute;
        style.inset.left = length(finite(offset_x));
        style.inset.top = length(finite(offset_y));
    }
    if node.width > 0.0 {
        style.size.width = length(safe_size(node.width));
    }
    if node.height > 0.0 {
        style.size.height = length(safe_size(node.height));
    }
    if node.min_width > 0.0 {
        style.min_size.width = length(safe_size(node.min_width));
    }
    if node.min_height > 0.0 {
        style.min_size.height = length(safe_size(node.min_height));
    }
    if node.gap > 0.0 {
        style.gap.width = length(safe_size(node.gap));
        style.gap.height = length(safe_size(node.gap));
    }
    if node.flex_grow > 0.0 {
        style.flex_grow = node.flex_grow;
    }
    style
}

#[habit]
pub struct Instance {}
impl Scope for Instance {
    fn is_enabled(&mut self, _ledger: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, _ledger: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all()
    }
}
impl Habit for Instance {
    fn tick(&mut self, ledger: &mut Ledger, context: &mut Context3D, _nerve: &mut Nerve) {
        // fill an unauthored viewport in the record itself, so the pass and drivers read the same value
        let gpu = &Services::get().gpu;
        let capture = Vector2::new(gpu.capture_width as f32, gpu.capture_height as f32);
        let real_size = ledger.read::<SysRecordUi>().screen_on(capture).pixels();
        if ledger.read::<SysRecordUi>().viewport_needs_fill(real_size) {
            ledger.write::<SysRecordUi>(|screen| {
                screen.fill_viewport(real_size);
            });
        }
        let state_ui = ledger.read::<SysRecordUi>();
        let viewport = state_ui.viewport;
        if !(viewport.x > 0.0 && viewport.y > 0.0) {
            ledger.log(Severity::Warning, &"ui_overlay: layout habit has no viewport to lay out against");
            return;
        }

        // this screen's cursor and click, for hover/click feedback
        let state_input = ledger.read::<SysRecordInput>();
        let cursor_raw = state_input.raw.get_axis(&AxisCode::Cursor).position;
        let clicked = state_input
            .raw
            .get_button(&ButtonCode::MousePrimary)
            .is_down;

        // the cursor arrives in capture pixels (winit window pixels, or the editor's
        // fitted image mapped back to frame pixels); no y flip, the pass maps the ui's top edge to the screen's top
        let mut cursor_space = state_ui.cursor_space;
        if !(cursor_space.x > 0.0 && cursor_space.y > 0.0) {
            let gpu = &Services::get().gpu;
            cursor_space = Vector2::new(gpu.capture_width as f32, gpu.capture_height as f32);
        }
        if !(cursor_space.x > 0.0 && cursor_space.y > 0.0) {
            cursor_space = viewport;
        }
        let cursor = Vector2::new(cursor_raw.x * (viewport.x / cursor_space.x), cursor_raw.y * (viewport.y / cursor_space.y));

        // snapshot every UI node. taffy runs against the snapshot because
        // the world cannot stay borrowed while the tree builds
        let mut entries: HashMap<i32, UiEntry> = HashMap::new();
        for node in context.get::<UiNode>() {
            let id = node.form().instance_id();
            if entries.contains_key(&id) {
                ledger.log(Severity::Warning, &format!("ui_overlay: form {} has a second UiNode — keeping the first", id));
                continue;
            }
            entries.insert(id, UiEntry::new(node));
        }
        if entries.is_empty() {
            ledger.write::<SysRecordUi>(|screen| screen.draw_calls.clear());
            return;
        }

        // walk each node up its form chain: nearest UI ancestor, pixel
        // offset through any plain forms, and how deep it is
        let ui_ids: HashSet<i32> = entries.iter().map(|(key, _)| *key).collect();
        for (_, entry) in entries.iter_mut() {
            let mut chain: Vec<Form> = Vec::new();
            let mut walker = entry.form.parent();
            while let Some(parent) = walker {
                chain.push(parent.clone());
                walker = parent.parent();
                if chain.len() >= 256 {
                    break;
                }
            }
            if chain.len() >= 256 {
                ledger.log(Severity::Error, &format!("ui_overlay: form chain past 256 parents — the composition has a cycle at form {}", entry.form.instance_id()));
                continue;
            }
            // the nearest UI ancestor adopts this node; deeper ones only
            // add to its depth
            let mut ancestor = chain.len();
            for index in 0..chain.len() {
                if ui_ids.contains(&chain[index].instance_id()) {
                    ancestor = index;
                    break;
                }
            }
            let mut depth: u32 = 0;
            for index in 0..chain.len() {
                if ui_ids.contains(&chain[index].instance_id()) {
                    depth += 1;
                }
            }
            entry.depth = depth;
            if ancestor < chain.len() {
                entry.parent_ui = Some(chain[ancestor].instance_id());
                if ancestor > 0 {
                    entry.top_id = Some(chain[ancestor - 1].instance_id());
                } else {
                    entry.top_id = Some(entry.form.instance_id());
                }
            }
            // the offset through every form between here and the UI parent — my
            // own position plus each plain form's on the way. Only my own turn and scale count
            let mut offset = Vector3::zero();
            if let Some(transform) = entry.form.get_facet::<Transform3D>() {
                offset = Vector3::new(finite(transform.position.x), finite(transform.position.y), finite(transform.position.z));
                entry.rotation = planar_angle(transform.rotation);
                entry.scale = Vector2::new(finite(transform.scale.x), finite(transform.scale.y));
            }
            let limit = if ancestor < chain.len() { ancestor } else { chain.len() };
            for form in chain.iter().take(limit) {
                if let Some(transform) = form.get_facet::<Transform3D>() {
                    offset.x += finite(transform.position.x);
                    offset.y += finite(transform.position.y);
                    offset.z += finite(transform.position.z);
                }
            }
            entry.offset_x = offset.x;
            entry.offset_y = offset.y;
            entry.offset_z = offset.z;
        }

        // direct UI children per node, in author order: a child is adopted
        // by this node when its top form is one of this node's children
        let mut ui_children: HashMap<i32, Vec<i32>> = HashMap::new();
        for (id, entry) in entries.iter() {
            let mut kids: Vec<i32> = Vec::new();
            for child in entry.form.children() {
                let child_id = child.instance_id();
                if let Some(adopted) = entries.get(&child_id)
                    && adopted.parent_ui == Some(*id)
                {
                    kids.push(child_id);
                }
            }
            ui_children.insert(*id, kids);
        }

        // build the taffy tree deepest first so a parent's children already
        // exist when it takes them
        let mut deepest_first: Vec<(u32, i32)> = Vec::new();
        for (id, entry) in entries.iter() {
            deepest_first.push((entry.depth, *id));
        }
        deepest_first.sort_by(|a, b| {
            if a.0 == b.0 {
                std::cmp::Ordering::Equal
            } else if b.0 > a.0 {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            }
        });

        let mut tree: TaffyTree<()> = TaffyTree::new();
        let mut taffy_nodes: HashMap<i32, NodeId> = HashMap::new();
        for pair in deepest_first.iter() {
            let id = pair.1;
            let mut kids: Vec<NodeId> = Vec::new();
            if let Some(child_ids) = ui_children.get(&id) {
                for child_id in child_ids {
                    if let Some(child_node) = taffy_nodes.get(child_id) {
                        kids.push(*child_node);
                    }
                }
            }
            if let Some(entry) = entries.get(&id) {
                let style = ui_style(&entry.node, entry.offset_x, entry.offset_y);
                let made = if kids.is_empty() { tree.new_leaf(style) } else { tree.new_with_children(style, &kids) };
                match made {
                    Ok(taffy_node) => {
                        taffy_nodes.insert(id, taffy_node);
                    }
                    Err(error) => {
                        ledger.log(Severity::Warning, &format!("ui_overlay: taffy rejected form {}: {}", id, error));
                    }
                }
            }
        }

        // lay every root subtree out against this screen. Roots hang off
        // viewport-space offsets, so nothing needs a camera here
        let root_space = Size {
            width: AvailableSpace::Definite(viewport.x),
            height: AvailableSpace::Definite(viewport.y),
        };
        for pair in deepest_first.iter() {
            let id = pair.1;
            if let Some(entry) = entries.get(&id)
                && entry.parent_ui.is_none()
                && let Some(node) = taffy_nodes.get(&id)
                && tree.compute_layout(*node, root_space).is_err()
            {
                ledger.log(Severity::Warning, &format!("ui_overlay: taffy layout failed at root form {}", id));
            }
        }

        // copy the boxes onto the screen, parents first so a child starts from
        // its parent's resolved box, turn, scale and layer
        let mut placed: HashMap<i32, (UiRect, f32, Vector2, f32)> = HashMap::new();
        let mut orders: HashMap<i32, u32> = HashMap::new();
        for pair in deepest_first.iter().rev() {
            let id = pair.1;
            if let Some(entry) = entries.get(&id) {
                if let Some(node) = taffy_nodes.get(&id) {
                    match tree.layout(*node) {
                        Ok(resolved) => {
                            orders.insert(id, resolved.order);
                            // a node that flexed to nothing is still worth
                            // drawing when it asked for a box
                            let mut width = safe_size(resolved.size.width);
                            let mut height = safe_size(resolved.size.height);
                            if width <= 0.0 {
                                width = safe_size(entry.node.width);
                            }
                            if height <= 0.0 {
                                height = safe_size(entry.node.height);
                            }
                            // taffy answers each node inside its upright parent, so carry
                            // that answer through the parent's own turn and scale
                            let mut parent = (UiRect::zero(), 0.0, Vector2::one(), 0.0);
                            if let Some(parent_id) = entry.parent_ui
                                && let Some(found) = placed.get(&parent_id)
                            {
                                parent = *found;
                            }
                            let (parent_rect, parent_rotation, parent_scale, parent_depth) = parent;
                            let local_center = Vector2::new(parent_rect.x + finite(resolved.location.x) + width * 0.5, parent_rect.y + finite(resolved.location.y) + height * 0.5);
                            let center = parent_rect.turn_point(local_center.x, local_center.y, parent_rotation, parent_scale);
                            let rect = UiRect::new(center.x - width * 0.5, center.y - height * 0.5, width, height);
                            let rotation = parent_rotation + entry.rotation;
                            let scale = Vector2::new(parent_scale.x * entry.scale.x, parent_scale.y * entry.scale.y);
                            let depth = parent_depth + entry.offset_z;
                            placed.insert(id, (rect, rotation, scale, depth));
                            // hit-test against the box as it will be drawn
                            let hovered = rect.contains_turned(cursor.x, cursor.y, rotation, scale);
                            entry.form.try_edit_facet::<UiNode>(|facet| {
                                facet.rect = rect;
                                facet.rotation = rotation;
                                facet.scale = scale;
                                facet.depth = depth;
                                facet.hovered = hovered;
                                facet.clicked = clicked && hovered;
                            });
                        }
                        Err(error) => {
                            ledger.log(Severity::Warning, &format!("ui_overlay: no layout for form {}: {}", id, error));
                        }
                    }
                }
            }
        }

        // fill the screen's draw calls for the overlay pass
        ledger.write::<SysRecordUi>(|screen| {
            screen.draw_calls.clear();
            for (id, entry) in entries.iter() {
                // panels only arrange — they never paint a box themselves
                if entry.node.kind == UiKind::Panel {
                    continue;
                }
                // read the box back so hover/click from this very tick is
                // what gets drawn
                if let Some(facet) = entry.form.get_facet::<UiNode>() {
                    if !(facet.rect.width > 0.0 && facet.rect.height > 0.0) {
                        continue;
                    }
                    let mut color = visible_color(facet.color);
                    if facet.clicked {
                        color = brighten(color, 1.3);
                    } else if facet.hovered {
                        color = brighten(color, 1.1);
                    }
                    let mut order: u32 = 0;
                    if let Some(found) = orders.get(id) {
                        order = *found;
                    }
                    screen
                        .draw_calls
                        .push(UiDrawCall::new(facet.rect, color, facet.kind, facet.depth, facet.text.clone(), facet.font_size, *id, order).turned(facet.rotation, facet.scale));
                }
            }
            // the authored depth wins, taffy's paint order breaks ties
            screen
                .draw_calls
                .sort_by(|a, b| a.depth.total_cmp(&b.depth).then(a.order.cmp(&b.order)));
        });
    }
}

#[test]
fn ui_nodes_author_as_a_taffy_tree() {
    use curio_core::Color;

    let world = std::rc::Rc::new(std::cell::RefCell::new(hecs::World::new()));
    let mut context = Context3D::new(world.clone());

    // a Panel arranges its children; taffy gives it the whole viewport
    let panel = context
        .spawner("hud")
        .facet(
            UiNode::builder()
                .gap(12.0)
                .min_size(400.0, 200.0)
                .absolute(true)
                .build(),
        )
        .spawn();

    // a Button paints a box and gets hover/click feedback for free
    let button = context
        .spawner("fire")
        .facet(
            UiNode::builder()
                .kind(UiKind::Button)
                .size(120.0, 32.0)
                .color(Color::new_01(0.8, 0.2, 0.2, 1.0))
                .text("fire")
                .font_size(16.0)
                .build(),
        )
        .spawn();

    // the builder does not attach children — set_parent is what wires the tree
    button.set_parent(Some(panel.clone()));

    // UI forms place with Transform3D, in pixels below-right of their UI parent
    let mut shift = Transform3D::default();
    shift.position = Vector3::new(40.0, 20.0, 1.0);
    // add_facet consumes and returns the form, so the handle is rebound
    let button = button.add_facet(shift);

    // the button hangs under the panel and owns exactly one UiNode
    let mut attached = false;
    if let Some(parent) = button.parent() {
        attached = parent.instance_id() == panel.instance_id();
    }
    assert!(attached);

    let mut authored = false;
    if let Some(node) = button.get_facet::<UiNode>() {
        authored = node.kind == UiKind::Button && node.width == 120.0 && node.height == 32.0 && !node.hovered && node.text == "fire" && node.font_size == 16.0;
    }
    assert!(authored);

    // the free-form flag survives onto the panel too: it hangs at its own
    // Transform3D offset instead of floating in flow
    let mut flagged = false;
    if let Some(node) = panel.get_facet::<UiNode>() {
        flagged = node.absolute;
    }
    assert!(flagged);

    let mut placed = false;
    if let Some(transform) = button.get_facet::<Transform3D>() {
        placed = transform.position.x == 40.0 && transform.position.y == 20.0;
    }
    assert!(placed);
}

#[test]
fn ui_text_follows_the_shared_font_grid() {
    use crate::{DEFAULT_FONT, UiFontDesc, UiFontGrid};

    // the same descriptor 3D text's FontAsset parses — ui_overlay must read
    // the identical grid off the identical atlas
    let Some(desc) = UiFontDesc::parse(DEFAULT_FONT) else {
        panic!("embedded default.font could not be parsed");
    };
    let grid = UiFontGrid::new(desc, 256, 128);
    assert_eq!(grid.columns, 13);
    assert_eq!(grid.rows, 9);

    // 'A' is the first char of char_order, so it owns the top-left cell
    match grid.glyph_uv('A') {
        Some([u_min, v_top, u_max, v_bottom]) => {
            assert!((u_min - 0.00390625).abs() < 0.00001, "u_min {}", u_min);
            assert!((v_top - 0.0078125).abs() < 0.00001, "v_top {}", v_top);
            assert!((u_max - 0.05078125).abs() < 0.00001, "u_max {}", u_max);
            assert!((v_bottom - 0.1015625).abs() < 0.00001, "v_bottom {}", v_bottom);
        }
        None => {
            panic!("'A' has no cell in the shared grid");
        }
    }

    // chars the atlas has no cell for resolve to None, so the pass skips
    // them rather than drawing garbage from a neighbour's texels
    match grid.glyph_uv('#') {
        Some(_) => panic!("'#' must not resolve in the default grid"),
        None => {}
    }
}

#[test]
fn nested_leaf_under_a_negative_id_anchor_keeps_its_ui_parent() {
    use curio_core::{AssetCache, AssetDatabase, AssetPipeline, CurioNetwork, CurioNetworkParticipant, Gpu, Logger};

    // the ledger logs through the Services singleton, so stand one up with no
    // gpu. The habit never touches the gpu while the record has a viewport
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

    let world = std::rc::Rc::new(std::cell::RefCell::new(hecs::World::new()));
    let mut context = Context3D::new(world.clone());

    // the panel root sits at the viewport origin
    let panel = context
        .spawner("hud")
        .facet(UiNode::builder().min_size(400.0, 200.0).build())
        .spawn();

    // instance ids are random and mostly negative. The bug read any negative
    // UI parent id as "no parent", so force the anchor onto a negative id:
    // keep spawning plain forms until one draws one, then make that the anchor
    let mut anchor = context.spawner("anchor").spawn();
    let mut tries = 0;
    while anchor.instance_id() >= 0 && tries < 1000 {
        anchor = context.spawner("anchor").spawn();
        tries += 1;
    }
    assert!(anchor.instance_id() < 0, "no negative instance id drawn in {} tries", tries);
    let anchor = anchor.add_facet(
        UiNode::builder()
            .kind(UiKind::Block)
            .size(200.0, 100.0)
            .build(),
    );
    anchor.set_parent(Some(panel.clone()));
    let mut anchor_shift = Transform3D::default();
    anchor_shift.position = Vector3::new(300.0, 200.0, 0.0);
    let anchor = anchor.add_facet(anchor_shift);

    // the leaf hangs under the anchor, inset by its own offset
    let leaf = context
        .spawner("plaque")
        .facet(
            UiNode::builder()
                .kind(UiKind::Block)
                .size(50.0, 20.0)
                .build(),
        )
        .spawn();
    leaf.set_parent(Some(anchor.clone()));
    let mut leaf_shift = Transform3D::default();
    leaf_shift.position = Vector3::new(10.0, 5.0, 0.0);
    let leaf = leaf.add_facet(leaf_shift);

    let network = CurioNetwork::new(vec![CurioNetworkParticipant::new(0, NetworkModes::LocalHost)], 0);
    let mut ledger = Ledger::new(network.clone());
    let mut nerve = Nerve::new(network);
    ledger.write::<SysRecordUi>(|screen| screen.viewport = Vector2::new(1280.0, 720.0));
    let mut habit = Instance {};
    habit.tick(&mut ledger, &mut context, &mut nerve);

    let mut anchor_rect = UiRect::new(-1.0, -1.0, 0.0, 0.0);
    if let Some(node) = anchor.get_facet::<UiNode>() {
        anchor_rect = node.rect;
    }
    assert_eq!((anchor_rect.x, anchor_rect.y), (300.0, 200.0), "anchor box {:?}", (anchor_rect.x, anchor_rect.y));

    // the leaf starts from the anchor's corner, not from the viewport origin
    let mut leaf_rect = UiRect::new(-1.0, -1.0, 0.0, 0.0);
    if let Some(node) = leaf.get_facet::<UiNode>() {
        leaf_rect = node.rect;
    }
    assert_eq!((leaf_rect.x, leaf_rect.y, leaf_rect.width, leaf_rect.height), (310.0, 205.0, 50.0, 20.0));
}
