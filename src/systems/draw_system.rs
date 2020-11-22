///
/// Draws graphics onto the screen.
/// 

use crate::{entity_manager::EntityManager, renderer::Renderer};

// Iterate over entities and update them.
pub fn update(entity_mgr: &mut EntityManager, renderer: &mut Renderer) {
    for entity_id in 0..entity_mgr.entities.len() {
        if entity_mgr.entities[entity_id].exists && entity_mgr.entities[entity_id].use_draw {
            let sprite_comp = entity_mgr.get_sprite_comp(entity_id as u32);

            if sprite_comp.should_update {
                sprite_comp.should_update = false;
                sprite_comp.quad_id = renderer.create_render_quad();
                let tex_id = renderer.load_texture(sprite_comp.tex_name.as_str());
                renderer.attach_tex_to_quad(sprite_comp.quad_id, tex_id);
            }

            let quad_id = sprite_comp.quad_id;
            let pos_comp = entity_mgr.get_pos_comp(entity_id as u32);
            renderer.set_quad_pos(quad_id, pos_comp.x, pos_comp.y);
        }
    }
}