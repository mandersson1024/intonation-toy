#![cfg(target_arch = "wasm32")]

use crate::web;

pub fn compensate_positions_for_canvas_scaling(events: &mut Vec<three_d::Event>, render_size: u32) {
    let canvas_style_size = web::utils::get_canvas_style_size();
    let render_size_f32 = render_size as f32;
    
    for event in events {
        match event {
            three_d::Event::MouseMotion { position, .. } |
            three_d::Event::MousePress { position, .. } | 
            three_d::Event::MouseRelease { position, .. } |
            three_d::Event::MouseWheel { position, .. } => {
                scale_event_position(position, render_size_f32, canvas_style_size);
            }
            _ => {}
        }
    }
}

fn scale_event_position(position: &mut three_d::PhysicalPoint, render_size: f32, canvas_style_size: f32) {
    let scale_factor = render_size / canvas_style_size;
    let offset = canvas_style_size - render_size;
    
    position.x *= scale_factor;
    position.y = (position.y + offset) * scale_factor;
}