#![cfg(target_arch = "wasm32")]

// External crate imports
use std::sync::Arc;
use three_d::{Camera, ClearState, Context, CpuTexture, Deg, Gm, Object, PhysicalPoint, RenderTarget, TextureData, Texture2DRef, Viewport};
use three_d::core::{DepthTexture2D, Interpolation, Texture2D, Wrapping};
use three_d::renderer::geometry::Rectangle;

use crate::app_config::{CLARITY_THRESHOLD, USER_PITCH_LINE_LEFT_MARGIN, USER_PITCH_LINE_RIGHT_MARGIN, OCTAVE_LINE_THICKNESS, REGULAR_LINE_THICKNESS};
use crate::presentation::audio_analysis::AudioAnalysis;
use crate::presentation::background_shader::BackgroundShaderMaterial;
use crate::presentation::egui_text_backend::EguiTextBackend;
use crate::presentation::tuning_lines::TuningLines;
use crate::presentation::user_pitch_line::UserPitchLine;
use crate::common::shared_types::{ColorScheme, MidiNote};
use crate::common::theme::{get_current_color_scheme, rgb_to_srgba_with_alpha};

/// Width of the data texture used for historical data
const DATA_TEXTURE_WIDTH: usize = 512;

/// Converts musical interval to screen Y position
fn interval_to_screen_y_position(interval: f32, viewport_height: f32) -> f32 {
    const ZOOM_FACTOR: f32 = 0.92;
    viewport_height * (0.5 + interval * ZOOM_FACTOR * 0.5)
}

/// Creates a textured quad for background rendering with custom shader
fn create_background_quad(
    context: &Context,
    width: u32,
    height: u32,
    texture: Texture2DRef,
    data_texture: Option<Texture2DRef>,
    data_index: i32,
) -> Gm<Rectangle, BackgroundShaderMaterial> {
    assert!(width > 0 && height > 0, "Dimensions must be positive: {}x{}", width, height);

    let (w, h) = (width as f32, height as f32);

    Gm::new(
        Rectangle::new(context, (w * 0.5, h * 0.5), Deg(0.0), w, h),
        BackgroundShaderMaterial {
            texture: Some(texture),
            data_texture,
            data_index,
            data_texture_width: DATA_TEXTURE_WIDTH as f32,
            left_margin: 0.1,
            right_margin: 0.1,
        }
    )
}

/// Calculates line thickness and alpha based on clarity value
fn calculate_pitch_line_appearance(clarity: Option<f32>) -> (f32, f32) {
    let Some(clarity_value) = clarity else {
        return (crate::app_config::USER_PITCH_LINE_THICKNESS_MAX, crate::app_config::USER_PITCH_LINE_TRANSPARENCY_MAX);
    };
    
    let normalized_clarity = (clarity_value.clamp(CLARITY_THRESHOLD, 1.0) - CLARITY_THRESHOLD) / (1.0 - CLARITY_THRESHOLD);
    
    (
        crate::app_config::USER_PITCH_LINE_THICKNESS_MAX + 
            normalized_clarity * (crate::app_config::USER_PITCH_LINE_THICKNESS_MIN - crate::app_config::USER_PITCH_LINE_THICKNESS_MAX),
        crate::app_config::USER_PITCH_LINE_TRANSPARENCY_MIN + 
            normalized_clarity * (crate::app_config::USER_PITCH_LINE_TRANSPARENCY_MAX - crate::app_config::USER_PITCH_LINE_TRANSPARENCY_MIN)
    )
}


pub struct Renderer {
    camera: Camera,
    user_pitch_line: UserPitchLine,
    audio_analysis: AudioAnalysis,
    tuning_lines: TuningLines,
    text_backend: EguiTextBackend,
    context: Context,
    color_scheme: ColorScheme,
    background_quad: Option<Gm<Rectangle, BackgroundShaderMaterial>>,
    presentation_context: Option<crate::common::shared_types::PresentationContext>,
    last_frame_time: f32,
    data_texture: Arc<Texture2D>,
    data_texture_index: usize,
    data_buffer: Vec<[f32; 2]>,
}

impl Renderer {
    pub fn new(context: &Context, viewport: Viewport) -> Result<Self, String> {
        let scheme = get_current_color_scheme();
        let tuning_lines = TuningLines::new(context, rgb_to_srgba_with_alpha(scheme.muted, 1.0));
        let text_backend = EguiTextBackend::new()?;

        // Create a 512x1 data texture that we'll write to incrementally
        let data_buffer = vec![[0.0_f32, 0.5_f32]; DATA_TEXTURE_WIDTH]; // Initialize all pixels
        let data_texture = Arc::new(Texture2D::new(
            context,
            &CpuTexture {
                data: TextureData::RgF32(data_buffer.clone()),
                width: DATA_TEXTURE_WIDTH as u32,
                height: 1,
                ..Default::default()
            },
        ));

        Ok(Self {
            camera: Camera::new_2d(viewport),
            user_pitch_line: UserPitchLine::default(),
            audio_analysis: AudioAnalysis::default(),
            tuning_lines,
            text_backend,
            context: context.clone(),
            color_scheme: scheme,
            background_quad: None,
            presentation_context: None,
            last_frame_time: 0.0,
            data_texture,
            data_texture_index: 0,
            data_buffer,
        })
    }

    
    fn refresh_colors(&mut self) {
        self.user_pitch_line.refresh_colors(&self.color_scheme, &self.audio_analysis);
        self.tuning_lines.clear();
    }
    
    /// Get tuning line positions for the active tuning system
    fn get_tuning_line_positions(&self, viewport: Viewport) -> Vec<(f32, MidiNote, f32)> {
        let Some(context) = &self.presentation_context else {
            return Vec::new();
        };
        
        let tuning_fork_frequency = crate::common::music_theory::midi_note_to_standard_frequency(context.tuning_fork_note);
        let mut line_data = Vec::new();
        
        for semitone in -12..=12 {
            if !crate::common::shared_types::semitone_in_scale(context.current_scale, semitone) {
                continue;
            }
            
            let y_position = if semitone == 0 {
                interval_to_screen_y_position(0.0, viewport.height as f32)
            } else {
                let frequency = crate::common::music_theory::interval_frequency(
                    context.tuning_system,
                    tuning_fork_frequency,
                    semitone,
                );
                let interval = (frequency / tuning_fork_frequency).log2();
                interval_to_screen_y_position(interval, viewport.height as f32)
            };
            
            let midi_note = (context.tuning_fork_note as i32 + semitone).clamp(0, 127) as MidiNote;
            let thickness = if semitone % 12 == 0 { OCTAVE_LINE_THICKNESS } else { REGULAR_LINE_THICKNESS };
            
            line_data.push((y_position, midi_note, thickness));
        }
        
        line_data
    }

    
    pub fn render(&mut self, screen: &mut RenderTarget, viewport: Viewport) {
        self.camera.set_viewport(viewport);
        
        let scheme = get_current_color_scheme();
        if scheme != self.color_scheme {
            self.color_scheme = scheme.clone();
            self.refresh_colors();
        }

        // Update time and render background quad with custom shader
        let delta_time = 1.0 / 60.0; // Simple frame time approximation (60 FPS assumed)
        self.last_frame_time += delta_time;

        if let Some(ref mut background_quad) = self.background_quad {
            // Update the data texture with detected and pitch values
            let detected = if self.audio_analysis.pitch_detected { 1.0 } else { 0.0 };
            let pitch = if self.audio_analysis.pitch_detected {
                let y_pos = interval_to_screen_y_position(self.audio_analysis.interval, viewport.height as f32);
                y_pos / viewport.height as f32
            } else {
                0.0
            };

            // Update the historical data buffer at current index
            let pixel_data = [detected, pitch];
            self.data_buffer[self.data_texture_index] = pixel_data;

            // Create new texture with the updated historical data
            self.data_texture = Arc::new(Texture2D::new(
                &self.context,
                &CpuTexture {
                    data: TextureData::RgF32(self.data_buffer.clone()),
                    width: DATA_TEXTURE_WIDTH as u32,
                    height: 1,
                    ..Default::default()
                },
            ));

            // Update the material with new texture and current index
            background_quad.material.data_texture = Some(self.data_texture.clone().into());            
            background_quad.material.data_index = self.data_texture_index as i32;

            // Increment index for next frame (wrap around)
            self.data_texture_index = (self.data_texture_index + 1) % DATA_TEXTURE_WIDTH;

            self.camera.disable_tone_and_color_mapping();
            screen.render(&self.camera, [background_quad], &[]);
            self.camera.set_default_tone_and_color_mapping();
        }

        if self.audio_analysis.pitch_detected {
            if let Some(ref mesh) = self.user_pitch_line.mesh() {
                screen.render(&self.camera, [mesh], &[]);
            }
        }
    }
    
    pub fn update_audio_analysis(&mut self, audio_analysis: AudioAnalysis) {
        self.audio_analysis = audio_analysis;
    }
    
    pub fn update_pitch_position(&mut self, viewport: Viewport) {
        if viewport.width == 0 || viewport.height == 0 {
            crate::common::dev_log!("Warning: Invalid viewport dimensions for pitch position update");
            return;
        }
        
        if !self.audio_analysis.pitch_detected {
            return;
        }
        
        let y = interval_to_screen_y_position(self.audio_analysis.interval, viewport.height as f32);
        let endpoints = (
            PhysicalPoint{x:USER_PITCH_LINE_LEFT_MARGIN, y}, 
            PhysicalPoint{x:viewport.width as f32 - USER_PITCH_LINE_RIGHT_MARGIN, y}
        );
        
        let (new_thickness, new_alpha) = calculate_pitch_line_appearance(self.audio_analysis.clarity);
        
        self.user_pitch_line.update_position(
            &self.context,
            endpoints,
            new_thickness,
            new_alpha,
            &self.color_scheme,
            &self.audio_analysis,
        );
    }
    
    pub fn update_tuning_lines(&mut self, viewport: Viewport, line_data: &[(f32, MidiNote, f32)]) {
        if viewport.width == 0 || viewport.height == 0 {
            crate::common::dev_log!("Warning: Invalid viewport dimensions for tuning lines update");
            return;
        }
        
        if line_data.is_empty() {
            crate::common::dev_log!("Warning: No tuning line data provided, clearing existing lines");
            self.tuning_lines.clear();
            return;
        }
        
        let scheme = get_current_color_scheme();
        self.tuning_lines.update_lines(viewport, line_data, &self.context, rgb_to_srgba_with_alpha(scheme.muted, 1.0));
    }
    
    /// Renders tuning lines and note labels to the background texture
    pub fn render_to_background_texture(&mut self, viewport: Viewport) {
        if viewport.width == 0 || viewport.height == 0 {
            crate::common::dev_log!("Warning: Invalid viewport dimensions for background texture");
            return;
        }
        
        let mut background_texture = Texture2D::new_empty::<[u8; 4]>(
            &self.context,
            viewport.width,
            viewport.height,
            Interpolation::Linear,
            Interpolation::Linear,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        
        let mut depth_texture = DepthTexture2D::new::<f32>(
            &self.context,
            viewport.width,
            viewport.height,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        
        {
            let camera = Camera::new_2d(viewport);
            let [r, g, b] = get_current_color_scheme().background;

            let tuning_lines: Vec<&dyn Object> = self.tuning_lines.lines().map(|line| line as &dyn Object).collect();
            let text_models = self.text_backend.render_texts(&self.context, viewport, &self.tuning_lines.get_note_labels());
            let text_objects: Vec<&dyn Object> = text_models.iter().map(|model| model.as_ref() as &dyn Object).collect();

            RenderTarget::new(background_texture.as_color_target(None), depth_texture.as_depth_target())
                .clear(ClearState::color_and_depth(r, g, b, 1.0, 1.0))
                .render(&camera, tuning_lines, &[])
                .render(&camera, text_objects, &[]);
        }
        
        let texture_ref = Texture2DRef::from_texture(background_texture);

        self.background_quad = Some(create_background_quad(
            &self.context,
            viewport.width,
            viewport.height,
            texture_ref,
            Some(self.data_texture.clone().into()),
            self.data_texture_index as i32
        ));
    }
    
    /// Update the presentation context
    pub fn update_presentation_context(&mut self, context: &crate::common::shared_types::PresentationContext, viewport: Viewport) {
        if self.presentation_context.as_ref() == Some(context) {
            return;
        }

        self.presentation_context = Some(context.clone());
        
        if viewport.width == 0 || viewport.height == 0 {
            crate::common::dev_log!("Warning: Invalid viewport dimensions for presentation context update");
            return;
        }
        
        let tuning_line_data = self.get_tuning_line_positions(viewport);
        self.update_tuning_lines(viewport, &tuning_line_data);
        self.render_to_background_texture(viewport);
    }
    
}

