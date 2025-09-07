#![cfg(target_arch = "wasm32")]

// External crate imports
use three_d::{Blend, Camera, ClearState, ColorMaterial, Context, Deg, Gm, Object, PhysicalPoint, RenderStates, RenderTarget, Srgba, Texture2DRef, Viewport, WriteMask};
use three_d::core::{DepthTexture2D, Interpolation, Texture2D, Wrapping};
use three_d::renderer::geometry::Rectangle;

use crate::app_config::{CLARITY_THRESHOLD, NOTE_LINE_LEFT_MARGIN, NOTE_LINE_RIGHT_MARGIN, OCTAVE_LINE_THICKNESS, REGULAR_LINE_THICKNESS};
use crate::presentation::audio_analysis::AudioAnalysis;
use crate::presentation::egui_text_backend::EguiTextBackend;
use crate::presentation::tuning_lines::TuningLines;
use crate::presentation::user_pitch_line::UserPitchLine;
use crate::common::shared_types::{ColorScheme, MidiNote};
use crate::common::theme::{get_current_color_scheme, rgb_to_srgba_with_alpha};

/// Converts musical interval to screen Y position
fn interval_to_screen_y_position(interval: f32, viewport_height: f32) -> f32 {
    const ZOOM_FACTOR: f32 = 0.92;
    viewport_height * (0.5 + interval * ZOOM_FACTOR * 0.5)
}

/// Creates a textured quad for background rendering
fn create_background_quad(
    context: &Context,
    width: u32,
    height: u32,
    texture: Texture2DRef,
) -> Gm<Rectangle, ColorMaterial> {
    assert!(width > 0 && height > 0, "Dimensions must be positive: {}x{}", width, height);
    
    let (w, h) = (width as f32, height as f32);
    
    Gm::new(
        Rectangle::new(context, (w * 0.5, h * 0.5), Deg(0.0), w, h),
        ColorMaterial {
            color: Srgba::WHITE,
            texture: Some(texture),
            is_transparent: true,
            render_states: RenderStates {
                depth_test: three_d::DepthTest::Always,
                write_mask: WriteMask::COLOR,
                blend: Blend::TRANSPARENCY,
                ..Default::default()
            },
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
    background_quad: Option<Gm<Rectangle, ColorMaterial>>,
    presentation_context: Option<crate::common::shared_types::PresentationContext>,
}

impl Renderer {
    pub fn new(context: &Context, viewport: Viewport) -> Result<Self, String> {
        let scheme = get_current_color_scheme();
        let tuning_lines = TuningLines::new(context, rgb_to_srgba_with_alpha(scheme.muted, 1.0));
        let text_backend = EguiTextBackend::new()?;

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

        if let Some(ref background_quad) = self.background_quad {
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
            PhysicalPoint{x:NOTE_LINE_LEFT_MARGIN, y}, 
            PhysicalPoint{x:viewport.width as f32 - NOTE_LINE_RIGHT_MARGIN, y}
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
        
        self.background_quad = Some(create_background_quad(
            &self.context, 
            viewport.width, 
            viewport.height, 
            Texture2DRef::from_texture(background_texture)
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

