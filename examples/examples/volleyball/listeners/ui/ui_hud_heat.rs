use curio_core::{
    AxisCode, ButtonCode, InputAxisState, KeyState, PrefabGameObject, Quaternion, Random, TextureAsset, Vector2, Vector3,
    collections::{event_queue::Nerve, ledger::Ledger},
    io::asset_loader::AssetLoader,
};
use std::collections::HashMap;

use gameplay::{
    built_in::facet::{
        animator::{animator_rotation_sin::AnimatorRotationSin, animator_scale_sin::AnimatorScaleSin},
        renderer::renderer_image::RendererImage,
        renderer_common::RendererCommon,
        transform::{transform2d::Transform2D, transform3d::Transform3D},
        tween::tween::{Tween, TweenCurve, TweenTransform2DPosition, TweenTransform2DRotation},
    },
    context_2d::Context2D,
    form::Form,
    traits::ui_panel::UIPanel,
    traits_internal::{ui_common::UICommon, world_context_common::ContextCommon},
};

use crate::{Assets, state::host::state_heat::StateHeat};

pub struct UIHUD {
    f_ui: Option<Form>,
    last_heat: i32,
}
impl UIHUD {
    pub fn new() -> Box<UIHUD> {
        Box::new(UIHUD { f_ui: None, last_heat: 0 })
    }
}
impl UIPanel for UIHUD {
    fn input_button(&mut self, _button: ButtonCode, _state: KeyState) {}
    fn input_axis(&mut self, _axis: AxisCode, _state: InputAxisState) {}
}
impl UICommon for UIHUD {
    fn init(&mut self) {}

    fn present(&mut self, ledger: &mut Ledger, _event_queue: &mut Nerve, context: &mut Context2D) {
        self.f_ui = Some(context.spawn_prefab_recursive(&AssetLoader::load_asset::<PrefabGameObject>(&Assets::PrefabUIHeat.into())));
    }

    fn dismiss(&mut self, _ledger: &mut Ledger, _event_queue: &mut Nerve, _context: &mut Context2D) {
        if let Some(ui) = &self.f_ui {
            ui.destroy();
        }
        self.f_ui = None;
    }

    fn tick(&mut self, ledger: &mut Ledger, _event_queue: &mut Nerve, context: &mut Context2D) {
        // get cur turn
        let cur_heat = ledger.read::<StateHeat>().all_players.clone();
        let heat = cur_heat.get(&ledger.network.me().guid).unwrap().clone();

        if let Some(ui) = &self.f_ui {
            ui.try_edit_facet_in_child::<Transform2D>("marker", |x| {
                let cur = x.position;
                let tar = (heat as f32 / 30.0).clamp(0.0, 1.0) * 0.5 + 0.2;
                x.position = Vector2::lerp(cur, Vector2::new(cur.x, tar), 0.2);
            });

            ui.try_edit_facets_in_child::<(RendererImage, AnimatorRotationSin, AnimatorScaleSin)>("topper", |(r, animr, anims)| {
                let enabled = heat >= 30;
                //
                r.set_enabled(enabled);
                animr.set_enabled(enabled);
                anims.set_enabled(enabled);
            });

            if heat != self.last_heat {
                self.spawn_particle(context, heat - self.last_heat);
                self.last_heat = heat;
            }
        }
    }
}
impl UIHUD {
    fn spawn_particle(&self, context: &mut Context2D, count: i32) {
        println!("spawn particle {}", count);

        for i in 0..count {
            let f = context
                .spawn("blob", Transform2D::default())
                .add_facet_default::<RendererImage>()
                .add_facet_default::<Tween>();

            f.edit_facets::<(Transform2D, RendererImage, Tween)>(|(transform, rend, tween)| {
                // edit transform
                transform.scale = Vector3::one() * 0.1;
                // edit asset
                rend.set_asset(Some(AssetLoader::load_asset::<TextureAsset>(&Assets::TextureHeatGuageFill.into())));
                // add tween
                tween.add_tween(
                    TweenTransform2DPosition::new(Vector2::one(), Vector2::new(0.1, 0.2))
                        .curve(TweenCurve::EaseOut)
                        .delay(i as f32 * 0.25)
                        .duration(Random::range_float(0.2, 0.6))
                        .on_complete(Box::new(|| print!("complted"))),
                );
            });
        }
    }
}
