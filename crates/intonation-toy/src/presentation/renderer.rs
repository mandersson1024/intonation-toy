#![cfg(target_arch = "wasm32")]

// External crate imports
use std::sync::Arc;
use three_d::{Camera, ClearState, Context, CpuTexture, Deg, Gm, Object, RenderTarget, TextureData, Texture2DRef, Viewport};
use three_d::core::{DepthTexture2D, Interpolation, Texture2D, Wrapping};
use three_d::renderer::geometry::Rectangle;

use crate::app_config::{NOTE_LINE_LEFT_MARGIN, NOTE_LINE_RIGHT_MARGIN, OCTAVE_LINE_THICKNESS, REGULAR_LINE_THICKNESS};
use crate::presentation::audio_analysis::AudioAnalysis;
use crate::presentation::background_shader::{BackgroundShaderMaterial, DATA_TEXTURE_WIDTH};
use crate::presentation::egui_text_backend::EguiTextBackend;
use crate::presentation::tuning_lines::{TuningLines, ColorMode};
use crate::common::shared_types::{ColorScheme, MidiNote};
use crate::common::theme::{get_current_color_scheme, rgb_to_srgba_with_alpha};

/// Converts musical interval to screen Y position
fn interval_to_screen_y_position(interval: f32, viewport_height: f32, display_range: &crate::common::shared_types::DisplayRange) -> f32 {
    let (zoom_factor, y_offset) = match display_range {
        crate::common::shared_types::DisplayRange::TwoOctaves => (0.92, 0.0),
        crate::common::shared_types::DisplayRange::OneFullOctave => (1.84, -0.46),
        crate::common::shared_types::DisplayRange::TwoHalfOctaves => (1.84, -0.077),
    };

    viewport_height * (0.5 + y_offset + interval * zoom_factor * 0.5)
}

/// Converts frequency to screen Y position
fn frequency_to_screen_y_position(frequency: f32, tonal_center_frequency: f32, viewport_height: f32, display_range: &crate::common::shared_types::DisplayRange) -> f32 {
    let interval = (frequency / tonal_center_frequency).log2();
    interval_to_screen_y_position(interval, viewport_height, display_range)
}

/// Creates a textured quad for background rendering with custom shader
fn create_background_quad(
    context: &Context,
    width: u32,
    height: u32,
    texture: Texture2DRef,
    highlight_texture: Texture2DRef,
    data_texture: Option<Texture2DRef>,
    tint_color: three_d::Vec3,
    current_pitch_color: three_d::Vec3,
    latest_cents_offset: f32,
) -> Gm<Rectangle, BackgroundShaderMaterial> {
    assert!(width > 0 && height > 0, "Dimensions must be positive: {}x{}", width, height);

    let (w, h) = (width as f32, height as f32);

    Gm::new(
        Rectangle::new(context, (w * 0.5, h * 0.5), Deg(0.0), w, h),
        BackgroundShaderMaterial {
            texture: Some(texture),
            highlight_texture: Some(highlight_texture),
            data_texture,
            left_margin: NOTE_LINE_LEFT_MARGIN / width as f32,
            right_margin: NOTE_LINE_RIGHT_MARGIN / width as f32,
            tint_color,
            current_pitch_color,
            latest_cents_offset,
        }
    )
}


pub struct Renderer {
    camera: Camera,
    audio_analysis: AudioAnalysis,
    text_backend: EguiTextBackend,
    three_d_context: Context,
    color_scheme: ColorScheme,
    background_quad: Option<Gm<Rectangle, BackgroundShaderMaterial>>,
    presentation_context: Option<crate::common::shared_types::PresentationContext>,
    last_frame_time: f32,
    data_texture: Arc<Texture2D>,
    data_buffer: Vec<[f32; 2]>,
}

impl Renderer {
    pub fn new(context: &Context, viewport: Viewport) -> Result<Self, String> {
        let scheme = get_current_color_scheme();
        let text_backend = EguiTextBackend::new()?;

        // Create a 512x1 data texture that we'll write to incrementally
        let data_buffer = vec![[0.0_f32, 0.5_f32]; DATA_TEXTURE_WIDTH]; // Initialize all pixels
        let data_texture = Arc::new(Texture2D::new(
            context,
            &CpuTexture {
                data: TextureData::RgF32(data_buffer.clone()),
                width: DATA_TEXTURE_WIDTH as u32,
                height: 1,
                wrap_s: Wrapping::ClampToEdge,
                wrap_t: Wrapping::ClampToEdge,
                min_filter: Interpolation::Nearest,
                mag_filter: Interpolation::Nearest,
                ..Default::default()
            },
        ));

        Ok(Self {
            camera: Camera::new_2d(viewport),
            audio_analysis: AudioAnalysis::default(),
            text_backend,
            three_d_context: context.clone(),
            color_scheme: scheme,
            background_quad: None,
            presentation_context: None,
            last_frame_time: 0.0,
            data_texture,
            data_buffer,
        })
    }

    /// Get tuning line positions for the active tuning system
    fn get_tuning_line_positions(&self, viewport: Viewport) -> Vec<(f32, MidiNote, f32, i32)> {
        let Some(context) = &self.presentation_context else {
            return Vec::new();
        };
        
        let tonal_center_frequency = crate::common::music_theory::midi_note_to_standard_frequency(context.tonal_center_note);
        let mut line_data = Vec::new();
        
        for semitone in -12..=12 {
            if !crate::common::shared_types::semitone_in_scale(context.current_scale, semitone) {
                continue;
            }
            
            let y_position = if semitone == 0 {
                interval_to_screen_y_position(0.0, viewport.height as f32, &context.display_range)
            } else {
                let frequency = crate::common::music_theory::interval_frequency(
                    context.tuning_system,
                    tonal_center_frequency,
                    semitone,
                );
                let interval = (frequency / tonal_center_frequency).log2();
                interval_to_screen_y_position(interval, viewport.height as f32, &context.display_range)
            };
            
            let midi_note = (context.tonal_center_note as i32 + semitone).clamp(0, 127) as MidiNote;
            let thickness = if semitone % 12 == 0 { OCTAVE_LINE_THICKNESS } else { REGULAR_LINE_THICKNESS };
            
            line_data.push((y_position, midi_note, thickness, semitone));
        }
        
        line_data
    }

    
    pub fn render(&mut self, screen: &mut RenderTarget, viewport: Viewport) {
        self.camera.set_viewport(viewport);

        // Update background shader margins if viewport changed
        if let Some(ref mut background_quad) = self.background_quad {
            background_quad.material.left_margin = NOTE_LINE_LEFT_MARGIN / viewport.width as f32;
            background_quad.material.right_margin = NOTE_LINE_RIGHT_MARGIN / viewport.width as f32;
        }

        // Update time and render background quad with custom shader
        let delta_time = 1.0 / 60.0; // Simple frame time approximation (60 FPS assumed)
        self.last_frame_time += delta_time;

        if let Some(ref mut background_quad) = self.background_quad {
            // Update the data texture with detected and pitch values
            let detected = if self.audio_analysis.pitch_detected { 1.0 } else { 0.0 };
            let pitch = if self.audio_analysis.pitch_detected {
                self.audio_analysis.frequency
            } else {
                0.0
            };

            // Shift buffer left and add new data at the end
            self.data_buffer.remove(0);
            self.data_buffer.push([detected, pitch]);

            // Convert frequencies to screen positions for texture data
            let texture_data: Vec<[f32; 2]> = if let Some(context) = &self.presentation_context {
                self.data_buffer.iter().map(|&[detected, frequency]| {
                    let screen_y = if detected > 0.0 {
                        let y_pos = frequency_to_screen_y_position(frequency, self.audio_analysis.tonal_center_frequency, viewport.height as f32, &context.display_range);
                        y_pos / viewport.height as f32
                    } else {
                        0.0
                    };
                    [detected, screen_y]
                }).collect()
            } else {
                vec![[0.0, 0.0]; DATA_TEXTURE_WIDTH]
            };

            // Create new texture with the updated historical data
            self.data_texture = Arc::new(Texture2D::new(
                &self.three_d_context,
                &CpuTexture {
                    data: TextureData::RgF32(texture_data),
                    width: DATA_TEXTURE_WIDTH as u32,
                    height: 1,
                    wrap_s: Wrapping::ClampToEdge,
                    wrap_t: Wrapping::ClampToEdge,
                    ..Default::default()
                },
            ));

            // Update the material with new texture and latest cents offset
            background_quad.material.data_texture = Some(self.data_texture.clone().into());
            background_quad.material.latest_cents_offset = self.audio_analysis.cents_offset;

            self.camera.disable_tone_and_color_mapping();
            screen.render(&self.camera, [background_quad], &[]);
            self.camera.set_default_tone_and_color_mapping();
        }
    }
    
    pub fn update_audio_analysis(&mut self, audio_analysis: AudioAnalysis) {
        self.audio_analysis = audio_analysis;
    }
    
    
    /// Renders tuning lines and note labels to the background texture
    pub fn render_to_background_texture(&mut self, viewport: Viewport) {
        if viewport.width == 0 || viewport.height == 0 {
            crate::common::dev_log!("Warning: Invalid viewport dimensions for background texture");
            return;
        }

        // Get the tuning line positions and create TuningLines on the fly
        let tuning_line_data = self.get_tuning_line_positions(viewport);
        if tuning_line_data.is_empty() {
            crate::common::dev_log!("Warning: No tuning line data available");
            return;
        }

        let scheme = get_current_color_scheme();
        let regular_color = rgb_to_srgba_with_alpha(scheme.muted, 1.0);
        let octave_color = rgb_to_srgba_with_alpha(scheme.secondary, 1.0);

        let mut tuning_lines = TuningLines::new(&self.three_d_context, regular_color);
        tuning_lines.update_lines(viewport, &tuning_line_data, &self.three_d_context, regular_color, octave_color);

        let mut background_texture = Texture2D::new_empty::<[u8; 4]>(
            &self.three_d_context,
            viewport.width,
            viewport.height,
            Interpolation::Linear,
            Interpolation::Linear,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );

        let mut highlight_texture = Texture2D::new_empty::<[u8; 4]>(
            &self.three_d_context,
            viewport.width,
            viewport.height,
            Interpolation::Linear,
            Interpolation::Linear,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );

        let mut depth_texture = DepthTexture2D::new::<f32>(
            &self.three_d_context,
            viewport.width,
            viewport.height,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );

        let mut depth_texture_highlight = DepthTexture2D::new::<f32>(
            &self.three_d_context,
            viewport.width,
            viewport.height,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );

        // Render normal background texture with theme colors
        {
            let camera = Camera::new_2d(viewport);
            let [r, g, b] = get_current_color_scheme().background;

            let tuning_lines_objects: Vec<&dyn Object> = tuning_lines.lines().map(|line| line as &dyn Object).collect();

            // Render note labels on the left
            let note_labels = tuning_lines.get_note_labels(ColorMode::Normal);
            let note_text_models = self.text_backend.render_texts(&self.three_d_context, viewport, &note_labels, three_d::egui::Align::LEFT);

            // Render interval labels on the right (right-aligned)
            let interval_labels = tuning_lines.get_interval_labels(viewport.width as f32, ColorMode::Normal);
            let interval_text_models = self.text_backend.render_texts(&self.three_d_context, viewport, &interval_labels, three_d::egui::Align::RIGHT);

            // Combine all text objects
            let mut text_objects: Vec<&dyn Object> = Vec::new();
            text_objects.extend(note_text_models.iter().map(|model| model.as_ref() as &dyn Object));
            text_objects.extend(interval_text_models.iter().map(|model| model.as_ref() as &dyn Object));

            RenderTarget::new(background_texture.as_color_target(None), depth_texture.as_depth_target())
                .clear(ClearState::color_and_depth(r, g, b, 1.0, 1.0))
                .render(&camera, tuning_lines_objects, &[])
                .render(&camera, text_objects, &[]);
        }

        // Render highlight texture with white text
        {
            let camera = Camera::new_2d(viewport);
            let [r, g, b] = get_current_color_scheme().background;

            // Create highlight lines for highlight texture
            let highlight_lines = tuning_lines.get_lines(&self.three_d_context, viewport.width as f32, ColorMode::Highlight);
            let highlight_lines_refs: Vec<&dyn Object> = highlight_lines.iter().map(|line| line.as_ref() as &dyn Object).collect();

            // Get labels with white color
            let highlight_note_labels = tuning_lines.get_note_labels(ColorMode::Highlight);
            let highlight_note_text_models = self.text_backend.render_texts(&self.three_d_context, viewport, &highlight_note_labels, three_d::egui::Align::LEFT);

            let highlight_interval_labels = tuning_lines.get_interval_labels(viewport.width as f32, ColorMode::Highlight);
            let highlight_interval_text_models = self.text_backend.render_texts(&self.three_d_context, viewport, &highlight_interval_labels, three_d::egui::Align::RIGHT);

            // Combine all highlight text objects
            let mut highlight_text_objects: Vec<&dyn Object> = Vec::new();
            highlight_text_objects.extend(highlight_note_text_models.iter().map(|model| model.as_ref() as &dyn Object));
            highlight_text_objects.extend(highlight_interval_text_models.iter().map(|model| model.as_ref() as &dyn Object));

            RenderTarget::new(highlight_texture.as_color_target(None), depth_texture_highlight.as_depth_target())
                .clear(ClearState::color_and_depth(r, g, b, 1.0, 1.0))
                .render(&camera, highlight_lines_refs, &[])
                .render(&camera, highlight_text_objects, &[]);
        }

        let texture_ref = Texture2DRef::from_texture(background_texture);
        let highlight_texture_ref = Texture2DRef::from_texture(highlight_texture);

        // Set tint color using theme primary color
        let [r, g, b] = self.color_scheme.primary;
        let tint_color = three_d::Vec3::new(r, g, b);

        // Set extension color using theme accent color
        let [ar, ag, ab] = self.color_scheme.accent;
        let extension_color = three_d::Vec3::new(ar, ag, ab);

        self.background_quad = Some(create_background_quad(
            &self.three_d_context,
            viewport.width,
            viewport.height,
            texture_ref,
            highlight_texture_ref,
            Some(self.data_texture.clone().into()),
            tint_color,
            extension_color,
            self.audio_analysis.cents_offset
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

        self.render_to_background_texture(viewport);
    }
    
}

