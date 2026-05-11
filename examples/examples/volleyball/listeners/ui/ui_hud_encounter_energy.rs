use animation::assets::model_asset_animated::ModelAssetAnimated;
use curio_core::{
    AxisCode, ButtonCode, InputAxisState, KeyState, Severity, Vector2, Vector3,
    collections::{event_queue::Nerve, ledger::Ledger},
    io::asset_loader::AssetLoader,
    log,
};
use std::collections::HashMap;

use gameplay::{
    built_in::facet::{renderer::renderer_dynamic::RendererDynamic, transform::transform2d::Transform2D},
    context_2d::Context2D,
    form::Form,
    traits::ui_panel::UIPanel,
    traits_internal::ui_common::UICommon,
};

use crate::{
    Assets,
    state::{
        state_energy::StateEnergy,
        state_teams::{StateTeamAssignments, Teams},
    },
};

pub struct UIHUD {
    go_energy_0: HashMap<i32, Vec<Form>>,
    // go_energy_1: Vec<GameObject>,
}
impl UIHUD {
    pub fn new() -> Box<UIHUD> {
        Box::new(UIHUD { go_energy_0: HashMap::new() })
    }
}
impl UIPanel for UIHUD {
    fn input_button(&mut self, _button: ButtonCode, _state: KeyState) {}
    fn input_axis(&mut self, _axis: AxisCode, _state: InputAxisState) {}
}
impl UICommon for UIHUD {
    fn init(&mut self) {}

    fn present(&mut self, ledger: &mut Ledger, _event_queue: &mut Nerve, context: &mut Context2D) {
        ledger.log(Severity::Info, "present hud counter");

        let asset = AssetLoader::load_asset::<ModelAssetAnimated>(&Assets::EnergyToken.into());
        // let x_offset = 0.15;
        let y_start = 0.75;
        let y_spacing = -0.05;

        // iterate over each team
        for user_guid in &ledger.read::<StateTeamAssignments>().team_assignments {
            // iterate over each memeber in team
            for i in 0..user_guid.1.len() {
                let mut x_pos = if *user_guid.0 == Teams::Red { 0.15 } else { 0.85 };
                if user_guid.1.len() > 1 {
                    if i == 0 {
                        x_pos -= 0.05;
                    } else {
                        x_pos += 0.05;
                    }
                }
                // iterate over total number of energy
                for j in 0..10 {
                    let mut r = RendererDynamic::default();
                    r.set_asset(Some(asset.clone()));
                    r.set_animation("add", false);

                    let mut rr = RendererDynamic::default();
                    rr.set_asset(Some(asset.clone()));
                    rr.set_animation("add", false);

                    //create
                    let go_0 = context
                        .spawn(
                            &format!("animated.energy_0_{}", j),
                            Transform2D::default()
                                .set_scale(Vector3::one() * 0.05)
                                .set_position_01(Vector2::new(x_pos, y_start + j as f32 * y_spacing)),
                        )
                        .add_facet(r);

                    //collect

                    let uid = user_guid.1[i];
                    if !self.go_energy_0.contains_key(&uid) {
                        self.go_energy_0.insert(uid, Vec::new());
                    }

                    if let Some(val) = self.go_energy_0.get_mut(&uid) {
                        val.push(go_0);
                    };
                }
            }
        }
    }

    fn dismiss(&mut self, _ledger: &mut Ledger, _event_queue: &mut Nerve, _context: &mut Context2D) {
        for x in &self.go_energy_0 {
            for go in x.1 {
                go.destroy();
            }
        }

        self.go_energy_0.clear();
    }

    fn tick(&mut self, ledger: &mut Ledger, _event_queue: &mut Nerve, _context: &mut Context2D) {
        let state_energy = ledger.read::<StateEnergy>();

        for user_uid in state_energy.all_players.clone() {
            let Some(user_gos) = self.go_energy_0.get(&user_uid.0) else {
                continue;
            };

            for i in 0..user_gos.len() {
                let is_enabled = (i as i32) < user_uid.1.0;
                user_gos[i].edit_facet::<RendererDynamic>(|x| {
                    x.set_animation(if is_enabled { "add" } else { "remove" }, false);
                })
            }
        }
        // let state_teams = ledger.get::<StateTeamAssignments>();

        // for t in state_teams.team_assignments {
        //     match t.0 {
        //         Teams::Red => {
        //             if let Some(e) = state_energy.all_players.get(&t.1[0]) {
        //                 for i in 0..self.go_energy_0.len() {
        //                     let go = &self.go_energy_0[i];
        //                     let is_enabled = i < e.0.try_into().unwrap();
        //                     go.edit_component::<RendererAnimated>(|x| {
        //                         x.set_animation(if is_enabled { "add" } else { "remove" }, false);
        //                     })
        //                 }
        //             }
        //         }
        //         Teams::Blue => {
        //             if let Some(e) = state_energy.all_players.get(&t.1[0]) {
        //                 for i in 0..self.go_energy_1.len() {
        //                     let go = &self.go_energy_1[i];
        //                     let is_enabled = i < e.0.try_into().unwrap();
        //                     go.edit_component::<RendererAnimated>(|x| {
        //                         x.set_animation(if is_enabled { "add" } else { "remove" }, false);
        //                     })
        //                 }
        //             }
        //         }
        //     }
        // }
    }
}
