use crate::egui_tools::EguiRenderer;
use crate::render_feature_2d::RenderFeature2D;
use curio_core::{
    built_in::record::{
        state_debug::StateDebug,
        state_gui::{GUIState, GuiElementTypes},
        state_gui_debug::GUIStateDebug,
    },
    collections::{event_queue::EventQueue, game_state::GameState},
    system_adapters::adapter_system_gpu::SystemGPU,
};
use egui::{Color32, Frame, Pos2, Ui};
use egui_wgpu::{
    ScreenDescriptor,
    wgpu::{self, CommandEncoder, SurfaceTexture},
};

pub struct RenderFeatureDrawUI {}
impl RenderFeatureDrawUI {
    pub fn new() -> Box<RenderFeatureDrawUI> {
        Box::new(RenderFeatureDrawUI {})
    }

    fn draw_all_ui(game_state: &mut GameState, system_event_queue: &mut EventQueue, output: &SurfaceTexture, encoder: &mut CommandEncoder, egui_renderer: &mut EguiRenderer) {
        let window = SystemGPU::get_window();
        let queue = SystemGPU::get_queue();
        let device = SystemGPU::get_device();
        let config = SystemGPU::get_config();
        let state_gui_debug = &game_state.get::<GUIStateDebug>();

        // start gui
        egui_renderer.begin_frame(&window);

        if game_state.get::<StateDebug>().is_inspecting {
            //
            let gui_window = &state_gui_debug.finalize(game_state);

            //
            let mut x = |ui: &mut Ui| {
                for element in &gui_window.children {
                    match &element.gui_type {
                        GuiElementTypes::Rectangle => todo!(),
                        GuiElementTypes::Ellipse => todo!(),
                        GuiElementTypes::Label(label_desc) => {
                            for (_text_style, _font_id) in ui.style_mut().text_styles.iter_mut() {
                                // font_id.size = label_desc.font_size // whatever size you want here
                            }
                            ui.colored_label(Color32::from_rgb(label_desc.color.as_r_0255() as u8, label_desc.color.as_g_0255() as u8, label_desc.color.as_b_0255() as u8), &label_desc.contents);
                            // ui.colored_label(Color32::from_rgb(255, 255, 255), "TEST");
                        }
                        GuiElementTypes::Button(button_desc) => {
                            let b = ui.button(&button_desc.contents);
                            if b.clicked() {
                                (button_desc.on_click)(game_state, system_event_queue);
                                println!("clicked");
                            }
                            if b.hovered() {}
                        }
                    };
                }
            };
            egui::Window::new(gui_window.instance_id.clone())
                .frame(Frame::new().fill(Color32::TRANSPARENT))
                .title_bar(false)
                .current_pos(Pos2::new(gui_window.position.x, gui_window.position.y))
                .show(egui_renderer.context(), &mut x);
        }

        let surface_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [config.width, config.height],
            pixels_per_point: 1.0, //window.as_ref().scale_factor() as f32 * scale_factor as f32,
        };

        egui_renderer.end_frame_and_draw(&device, &queue, encoder, &window, &surface_view, screen_descriptor);
    }
}
impl RenderFeature2D for RenderFeatureDrawUI {
    fn render(&mut self, game_state: &mut GameState, system_event_queue: &mut EventQueue, output: &SurfaceTexture, encoder: &mut CommandEncoder, egui_renderer: &mut EguiRenderer) {
        RenderFeatureDrawUI::draw_all_ui(game_state, system_event_queue, output, encoder, egui_renderer);
    }

    fn clear(&mut self, game_state: &mut GameState) {
        game_state.edit::<GUIState>(|x| {
            x.guis.clear();
        });
        game_state.edit::<GUIStateDebug>(|x| {
            x.clear();
        });
    }
}
