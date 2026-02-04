use curio_core::Color;

use crate::{
    built_in::facet::renderer::{renderer_dynamic::RendererDynamic, renderer_static::RendererStatic, renderer_text::RendererText},
    context_3d::Context3D,
    form::Form,
    traits::facet_common::FacetCommon,
    traits_internal::world_context_common::ContextCommon,
};
use std::collections::{HashMap, VecDeque};

pub trait RendererCommon: FacetCommon {
    fn set_cached_enabled_in_hierarchy(&mut self, val: bool);
    fn get_cached_enabled_in_hierarchy(&self) -> bool;

    fn set_cached_tint_in_hierarchy(&mut self, val: Color);
    fn get_cached_tint_in_hierarchy(&self) -> Color;

    // hierachy
    // fn set_parent(&mut self, parent: Option<Form>);
    // fn get_parent(&self) -> Option<Form>;
    // tint
    fn set_tint(&mut self, tint: Color);
    fn get_tint(&self) -> Color;
    // enabled
    fn set_enabled(&mut self, enabled: bool);
    fn get_enabled(&self) -> bool;
    //
    // fn tint_in_hierachy(&self, world: &World) -> Color {
    //     let mut tint = self.get_tint();
    //     let mut current = self.get_parent();
    //     while let Some(parent_entity) = &current {
    //         // if let Some(parent_renderer) = parent_entity.get_component::<&ComponentRendererText>() {
    //         //     tint = tint * parent_renderer.get_tint();
    //         //     current = parent_renderer.get_parent();
    //         // } else if let Some(parent_renderer) = parent_entity.get_component::<&Renderer>() {
    //         //     tint = tint * parent_renderer.get_tint();
    //         //     current = parent_renderer.get_parent();
    //         // } else if let Some(parent_renderer) = parent_entity.get_component::<&RendererAnimated>() {
    //         //     tint = tint * parent_renderer.get_tint();
    //         //     current = parent_renderer.get_parent();
    //         // }
    //         if let Some(parent_renderer) = parent_entity.get_component::<ComponentRendererText>() {
    //             tint = tint * parent_renderer.get_tint();
    //             current = parent_renderer.get_parent();
    //         } else if let Some(parent_renderer) = parent_entity.get_component::<Renderer>() {
    //             tint = tint * parent_renderer.get_tint();
    //             current = parent_renderer.get_parent();
    //         } else if let Some(parent_renderer) = parent_entity.get_component::<RendererAnimated>() {
    //             tint = tint * parent_renderer.get_tint();
    //             current = parent_renderer.get_parent();
    //         }
    //     }

    //     return tint;
    // }

    fn update_tint_in_heirarchy(&self, _w: Context3D) {
        // let b = w.world.borrow();

        // let x = b.get::<&ComponentRendererText>(self.get_parent().unwrap().entity);

        // let mut tint = self.get_tint();
        // let mut current = self.get_parent();
        // while let Some(parent_entity) = &current {
        //     if let Some(parent_renderer) = parent_entity.get_component::<ComponentRendererText>() {
        //         tint = tint * parent_renderer.get_cached_tint_in_hierarchy();
        //         current = parent_renderer.get_parent();
        //     } else if let Some(parent_renderer) = parent_entity.get_component::<Renderer>() {
        //         tint = tint * parent_renderer.get_cached_tint_in_hierarchy();
        //         current = parent_renderer.get_parent();
        //     } else if let Some(parent_renderer) = parent_entity.get_component::<RendererAnimated>() {
        //         tint = tint * parent_renderer.get_cached_tint_in_hierarchy();
        //         current = parent_renderer.get_parent();
        //     }
        // }

        // self.set_cached_tint_in_hierarchy(tint);
    }
    // fn update(world: &mut Context3D) {
    //     let borrow = world.world.borrow_mut();
    //     for w in borrow.iter() {
    //         if let Some(mut x) = w.get::<&mut RendererText>() {
    //             _ = x.update_enabled_in_heirarchy(&borrow);
    //         }
    //         if let Some(mut x) = w.get::<&mut RendererDynamic>() {
    //             _ = x.update_enabled_in_heirarchy(&borrow);
    //         }
    //         if let Some(mut x) = w.get::<&mut RendererStatic>() {
    //             _ = x.update_enabled_in_heirarchy(&borrow);
    //         }
    //     }
    // }
    // fn update_enabled_in_heirarchy(&mut self, world: &RefMut<'_, World>) -> bool {
    //     let is_enabled = self.get_enabled();

    //     if let Some(parent_entity) = &self.form().parent() {
    //         let mut parent_is_enabled = false;
    //         if let Ok(mut parent_renderer) = world.get::<&mut RendererText>(parent_entity.entity()) {
    //             parent_is_enabled = parent_renderer.update_enabled_in_heirarchy(world);
    //         }
    //         if let Ok(mut parent_renderer) = world.get::<&mut RendererStatic>(parent_entity.entity()) {
    //             parent_is_enabled = parent_renderer.update_enabled_in_heirarchy(world);
    //         }
    //         if let Ok(mut parent_renderer) = world.get::<&mut RendererDynamic>(parent_entity.entity()) {
    //             parent_is_enabled = parent_renderer.update_enabled_in_heirarchy(world);
    //         }

    //         self.set_cached_enabled_in_hierarchy(is_enabled && parent_is_enabled);
    //         return is_enabled && parent_is_enabled;
    //     } else {
    //         self.set_cached_enabled_in_hierarchy(is_enabled);
    //         return is_enabled;
    //     }
    // }

    // fn enabled_in_hierarchy(&self, world: &WorldContext) -> bool {
    //     if !self.get_enabled() {
    //         return false;
    //     }

    //     let mut current = self.get_parent();
    //     while let Some(parent_entity) = &current {
    //         if let Some(parent_renderer) = parent_entity.get_component::<ComponentRendererText>() {
    //             if !parent_renderer.get_enabled() {
    //                 return false;
    //             } else {
    //                 current = parent_renderer.get_parent();
    //             }
    //         } else if let Some(parent_renderer) = parent_entity.get_component::<Renderer>() {
    //             if !parent_renderer.get_enabled() {
    //                 return false;
    //             } else {
    //                 current = parent_renderer.get_parent();
    //             }
    //         } else if let Some(parent_renderer) = parent_entity.get_component::<RendererAnimated>() {
    //             if !parent_renderer.get_enabled() {
    //                 return false;
    //             } else {
    //                 current = parent_renderer.get_parent();
    //             }
    //         } else {
    //             current = None;
    //         }
    //     }

    //     return true;
    // }
}
pub fn update_enabled(context: &Context3D) {
    let mut output: HashMap<Form, bool> = HashMap::new();
    let mut root: Vec<Form> = Vec::new();
    let mut all: HashMap<Form, bool> = HashMap::new();
    for x in context.get::<RendererDynamic>() {
        let form = x.form();
        let enabled = x.get_enabled();

        if form.parent().is_none() {
            root.push(form.clone());
        }
        all.insert(form, enabled);
    }
    for x in context.get::<RendererStatic>() {
        let form = x.form();
        let enabled = x.get_enabled();

        if form.parent().is_none() {
            root.push(form.clone());
        }
        all.insert(form, enabled);
    }
    for x in context.get::<RendererText>() {
        let form = x.form();
        let enabled = x.get_enabled();

        if form.parent().is_none() || !form.parent().unwrap().has_facet::<RendererText>() {
            root.push(form.clone());
        }
        all.insert(form, enabled);
    }
    for form in root {
        // get matrix for this entry
        let parent_matrix = all[&form];

        // add all children in starting point to queue
        let mut child_queue: VecDeque<(Form, bool)> = VecDeque::new();
        for child in form.children() {
            child_queue.push_back((child, parent_matrix));
        }

        // add to ouput
        output.insert(form, parent_matrix);

        // while there are children still in the queue
        while let Some((cur_form, parent_ws)) = child_queue.pop_front() {
            // calculate this matrix
            let matrix_parent = parent_ws;
            let matrix_self = all[&cur_form];
            let ws_matrix = matrix_parent && matrix_self;

            // add all children to the queue
            let children = cur_form.children();
            for child in children {
                child_queue.push_back((child, ws_matrix));
            }

            // add this to the output
            output.insert(cur_form, ws_matrix);
        }
    }

    for (form, enabled) in output {
        if form.has_facet::<RendererDynamic>() {
            //
            form.edit_facet::<RendererDynamic>(|x| {
                x.set_cached_enabled_in_hierarchy(enabled);
            });
        };
        if form.has_facet::<RendererStatic>() {
            //
            form.edit_facet::<RendererStatic>(|x| {
                x.set_cached_enabled_in_hierarchy(enabled);
            });
        };
        if form.has_facet::<RendererText>() {
            //
            form.edit_facet::<RendererText>(|x| {
                x.set_cached_enabled_in_hierarchy(enabled);
            });
        };
    }
}
