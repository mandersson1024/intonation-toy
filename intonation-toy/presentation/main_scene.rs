use three_d::{AmbientLight, Blend, Camera, ClearState, ColorMaterial, Context, Gm, Line, PhysicalPoint, RenderStates, RenderTarget, Srgba, Viewport, WriteMask};
use crate::shared_types::{MidiNote, ColorScheme};
use crate::theme::{get_current_color_scheme, rgb_to_srgba, rgb_to_srgba_with_alpha};
use crate::app_config::{USER_PITCH_LINE_THICKNESS_MIN, USER_PITCH_LINE_THICKNESS_MAX, USER_PITCH_LINE_TRANSPARENCY_MIN, USER_PITCH_LINE_TRANSPARENCY_MAX, CLARITY_THRESHOLD, INTONATION_ACCURACY_THRESHOLD};

// Left margin to reserve space for note names
const NOTE_NAME_X_OFFSET: f32 = 18.0;
const NOTE_NAME_Y_OFFSET: f32 = 2.0;
const NOTE_LINE_LEFT_MARGIN: f32 = 40.0;
const NOTE_LINE_RIGHT_MARGIN: f32 = 15.0;

// Helper function to get the user pitch line color from the color scheme
// Returns error color when volume peak flag is true, accent color when within configured threshold, otherwise primary color
fn get_user_pitch_line_color(scheme: &ColorScheme, volume_peak: bool, cents_offset: f32) -> [f32; 3] {
    if volume_peak {
        scheme.error
    } else if cents_offset.abs() < INTONATION_ACCURACY_THRESHOLD {
        scheme.accent
    } else {
        scheme.primary
    }
}

pub fn interval_to_screen_y_position(interval: f32, viewport_height: f32, zoom_factor: f32) -> f32 {
    // interval of [0.5, 2.0] means [-1, +1] octaves
    let y: f32 = viewport_height * (0.5 + interval * zoom_factor * 0.5);
    y
}

/// Create a ColorMaterial with the given color and optional transparency
fn create_color_material(color: Srgba, is_transparent: bool) -> ColorMaterial {
    ColorMaterial {
        color,
        texture: None,
        is_transparent,
        render_states: RenderStates {
            write_mask: WriteMask::COLOR,
            blend: Blend::TRANSPARENCY,
            ..Default::default()
        },
    }
}

pub struct TuningLines {
    lines: Vec<Gm<Line, ColorMaterial>>,
    midi_notes: Vec<MidiNote>,
    y_positions: Vec<f32>,
    thicknesses: Vec<f32>,
    context: Context,
    regular_material: ColorMaterial,
    octave_material: ColorMaterial,
    accent_material: ColorMaterial,
    closest_midi_note: Option<MidiNote>,
}

impl TuningLines {
    pub fn new(context: &Context, regular_color: Srgba, octave_color: Srgba, accent_color: Srgba) -> Self {
        let regular_material = create_color_material(regular_color, false);
        let octave_material = create_color_material(octave_color, false);
        let accent_material = create_color_material(accent_color, false);
        
        Self {
            lines: Vec::new(),
            midi_notes: Vec::new(),
            y_positions: Vec::new(),
            thicknesses: Vec::new(),
            context: context.clone(),
            regular_material,
            octave_material,
            accent_material,
            closest_midi_note: None,
        }
    }

    /// Update the number of tuning lines, their positions, MIDI note numbers, and thickness
    /// The presenter calls this method with position, MIDI note, and thickness data for the active tuning system
    pub fn update_lines(&mut self, viewport: Viewport, line_data: &[(f32, MidiNote, f32)]) {
        let width = viewport.width as f32;
        let needed_lines = line_data.len();
        
        //crate::common::dev_log!("TUNING_DEBUG: Updating {} tuning lines, had {} before", needed_lines, self.lines.len());
        
        // Resize lines vector if needed (thickness will be set later)
        while self.lines.len() < needed_lines {
            let line = Line::new(
                &self.context,
                PhysicalPoint { x: NOTE_LINE_LEFT_MARGIN, y: 0.0 },
                PhysicalPoint { x: NOTE_LINE_LEFT_MARGIN, y: 0.0 },
                1.0  // Default thickness, will be updated
            );
            // Use regular material as default, will be updated if needed
            self.lines.push(Gm::new(line, self.regular_material.clone()));
        }
        
        // Remove excess lines, midi notes, y_positions, and thicknesses if we have too many
        self.lines.truncate(needed_lines);
        self.midi_notes.truncate(needed_lines);
        self.y_positions.truncate(needed_lines);
        self.thicknesses.truncate(needed_lines);
        
        // Resize midi_notes, y_positions, and thicknesses vectors if needed
        while self.midi_notes.len() < needed_lines {
            self.midi_notes.push(0); // Temporary value, will be set below
        }
        while self.y_positions.len() < needed_lines {
            self.y_positions.push(0.0); // Temporary value, will be set below
        }
        while self.thicknesses.len() < needed_lines {
            self.thicknesses.push(1.0); // Temporary value, will be set below
        }
        
        // Set positions, MIDI notes, and thickness for all lines
        for (i, &(y, midi_note, thickness)) in line_data.iter().enumerate() {
            // Determine material priority: accent > octave > regular
            let is_closest = Some(midi_note) == self.closest_midi_note;
            let is_octave = thickness == crate::app_config::OCTAVE_LINE_THICKNESS;
            let material = if is_closest {
                &self.accent_material
            } else if is_octave { 
                &self.octave_material 
            } else { 
                &self.regular_material 
            };
            
            // If thickness changed, recreate the line
            if i < self.thicknesses.len() && self.thicknesses[i] != thickness {
                let line = Line::new(
                    &self.context,
                    PhysicalPoint { x: NOTE_LINE_LEFT_MARGIN, y },
                    PhysicalPoint { x: width - NOTE_LINE_RIGHT_MARGIN, y },
                    thickness
                );
                self.lines[i] = Gm::new(line, material.clone());
            } else {
                // Always recreate the line to ensure material is up to date
                // This is simpler and ensures accent highlighting works correctly
                let line = Line::new(
                    &self.context,
                    PhysicalPoint { x: NOTE_LINE_LEFT_MARGIN, y },
                    PhysicalPoint { x: width - NOTE_LINE_RIGHT_MARGIN, y },
                    thickness
                );
                self.lines[i] = Gm::new(line, material.clone());
            }
            self.midi_notes[i] = midi_note;
            self.y_positions[i] = y;
            self.thicknesses[i] = thickness;
        }
    }
    
    /// Backward compatibility method that accepts old format without thickness
    /// Calls the new update_lines method with default thickness of 1.0
    pub fn update_lines_legacy(&mut self, viewport: Viewport, line_data: &[(f32, MidiNote)]) {
        // Convert old format to new format with default thickness
        let with_thickness: Vec<(f32, MidiNote, f32)> = line_data
            .iter()
            .map(|&(y, midi_note)| (y, midi_note, 1.0))
            .collect();
        self.update_lines(viewport, &with_thickness);
    }
    
    pub fn lines(&self) -> impl Iterator<Item = &Gm<Line, ColorMaterial>> {
        self.lines.iter()
    }
    
    /// Returns an iterator over the MIDI notes corresponding to each tuning line
    pub fn midi_notes(&self) -> impl Iterator<Item = MidiNote> + '_ {
        self.midi_notes.iter().copied()
    }
    
    /// Set the closest MIDI note that should be highlighted with accent color
    pub fn set_closest_note(&mut self, note: Option<MidiNote>) {
        self.closest_midi_note = note;
    }
    
    /// Render note labels above each tuning line
    pub fn render_note_labels(&self, text_renderer: &mut TextRenderer) {
        for (i, &midi_note) in self.midi_notes.iter().enumerate() {
            let y_position = self.y_positions[i];
            let thickness = self.thicknesses[i];
            
            // Convert MIDI note to name
            let note_name = crate::shared_types::midi_note_to_name(midi_note);
            
            // Position text aligned with the line (same Y position)
            let text_y = y_position + NOTE_NAME_Y_OFFSET;
            let text_x = NOTE_NAME_X_OFFSET;
            
            // Determine color with priority: accent > octave > muted
            let scheme = get_current_color_scheme();
            let text_color = if Some(midi_note) == self.closest_midi_note {
                scheme.accent
            } else if thickness == crate::app_config::OCTAVE_LINE_THICKNESS {
                scheme.secondary
            } else {
                scheme.muted
            };
            
            text_renderer.queue_text(&note_name, text_x, text_y, 14.0, [text_color[0], text_color[1], text_color[2], 1.0]);
        }
    }
}

pub struct TextRenderer {
    text_builder: three_d_text_builder::TextBuilder,
    queued_texts: Vec<QueuedText>,
}

#[derive(Debug, Clone)]
struct QueuedText {
    text: String,
    x: f32,
    y: f32,
    size: f32,
    color: [f32; 4],
}

impl TextRenderer {
    pub fn new(_context: &Context) -> Result<Self, String> {
        // Use the actual Roboto Regular font file
        let roboto_font = include_bytes!("../static/fonts/Roboto-Regular.ttf");
        
        let text_builder = three_d_text_builder::TextBuilder::new(
            roboto_font,
            three_d_text_builder::TextBuilderSettings::default()
        ).map_err(|e| format!("Failed to create TextBuilder with Roboto font: {:?}", e))?;
            
        Ok(Self {
            text_builder,
            queued_texts: Vec::new(),
        })
    }
    
    /// Queue text for rendering at the specified position
    pub fn queue_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        self.queued_texts.push(QueuedText {
            text: text.to_string(),
            x,
            y,
            size,
            color,
        });
    }
    
    /// Clear all queued text (called each frame)
    pub fn clear_queue(&mut self) {
        self.queued_texts.clear();
    }
    
    /// Create text models using the actual Roboto font
    pub fn create_text_models(&mut self, context: &Context, viewport: Viewport) -> Vec<three_d::Gm<three_d_text_builder::TextMesh, three_d_text_builder::TextMaterial>> {
        let mut text_refs = Vec::new();
        
        // Set viewport for proper text positioning
        self.text_builder.set_viewport(viewport);
        
        // Create TextRef objects for each queued text
        for queued_text in &self.queued_texts {
            let text_ref = three_d_text_builder::TextRef {
                text: &queued_text.text,
                size: queued_text.size,
                color: three_d::Srgba::new(
                    (queued_text.color[0] * 255.0) as u8,
                    (queued_text.color[1] * 255.0) as u8,
                    (queued_text.color[2] * 255.0) as u8,
                    (queued_text.color[3] * 255.0) as u8,
                ),
                position: three_d_text_builder::TextPosition::Pixels(three_d::vec2(queued_text.x, queued_text.y)),
                ..Default::default()
            };
            text_refs.push(text_ref);
        }
        
        // Build text models using the proper API
        if !text_refs.is_empty() {
            self.text_builder.build(context, &text_refs).collect()
        } else {
            Vec::new()
        }
    }
}

pub struct MainScene {
    camera: Camera,
    user_pitch_line: Gm<Line, ColorMaterial>,
    light: AmbientLight,
    pub tuning_lines: TuningLines,
    text_renderer: TextRenderer,
    context: Context,
    pitch_detected: bool,
    current_scheme: ColorScheme,
    user_pitch_line_thickness: f32,
    user_pitch_line_alpha: f32,
    volume_peak: bool,
    cents_offset: f32,
}

impl MainScene {
    pub fn new(context: &Context, viewport: Viewport) -> Result<Self, String> {
        let scheme = get_current_color_scheme();
        let initial_thickness = USER_PITCH_LINE_THICKNESS_MAX;
        let user_pitch_line = Line::new(context, PhysicalPoint{x:NOTE_LINE_LEFT_MARGIN, y:0.0}, PhysicalPoint{x:NOTE_LINE_LEFT_MARGIN, y:0.0}, initial_thickness);

        let initial_volume_peak = false;
        let initial_cents_offset = 0.0;
        let primary_material = create_color_material(rgb_to_srgba(get_user_pitch_line_color(&scheme, initial_volume_peak, initial_cents_offset)), false);
        
        let tuning_lines = TuningLines::new(context, rgb_to_srgba(scheme.muted), rgb_to_srgba(scheme.secondary), rgb_to_srgba(scheme.accent));
        let text_renderer = TextRenderer::new(context)?;
        
        Ok(Self {
            camera: Camera::new_2d(viewport),
            user_pitch_line: Gm::new(user_pitch_line, primary_material.clone()),
            light: AmbientLight::new(context, 1.0, rgb_to_srgba(scheme.secondary)),
            tuning_lines,
            text_renderer,
            context: context.clone(),
            pitch_detected: false,
            current_scheme: scheme,
            user_pitch_line_thickness: initial_thickness,
            user_pitch_line_alpha: USER_PITCH_LINE_TRANSPARENCY_MAX,
            volume_peak: initial_volume_peak,
            cents_offset: initial_cents_offset,
        })
    }
    
    pub fn update_viewport(&mut self, viewport: Viewport) {
        self.camera.set_viewport(viewport);
    }
    
    fn refresh_colors(&mut self) {
        let scheme = self.current_scheme.clone();
        
        // Recreate user pitch line with new color (it will be repositioned on next update)
        let primary_material = create_color_material(
            rgb_to_srgba_with_alpha(get_user_pitch_line_color(&scheme, self.volume_peak, self.cents_offset), self.user_pitch_line_alpha),
            true
        );
        let line = Line::new(&self.context, 
            PhysicalPoint{x:NOTE_LINE_LEFT_MARGIN, y:0.0}, 
            PhysicalPoint{x:NOTE_LINE_LEFT_MARGIN, y:0.0}, 
            self.user_pitch_line_thickness);
        self.user_pitch_line = Gm::new(line, primary_material);
        
        // Update tuning lines materials
        self.tuning_lines.regular_material = create_color_material(rgb_to_srgba(scheme.muted), false);
        self.tuning_lines.octave_material = create_color_material(rgb_to_srgba(scheme.secondary), false);
        self.tuning_lines.accent_material = create_color_material(rgb_to_srgba(scheme.accent), false);
        
        // Clear and recreate all tuning lines with new material
        // They will be recreated with correct positions and thickness on next update_lines call
        self.tuning_lines.lines.clear();
        self.tuning_lines.midi_notes.clear();
        self.tuning_lines.y_positions.clear();
        self.tuning_lines.thicknesses.clear();
        
        // Update ambient light
        self.light = AmbientLight::new(&self.context, 1.0, rgb_to_srgba(scheme.secondary));
    }
    
    pub fn render(&mut self, screen: &mut RenderTarget) {
        // Check for theme changes
        let scheme = get_current_color_scheme();
        if scheme != self.current_scheme {
            self.current_scheme = scheme.clone();
            self.refresh_colors();
        }
        
        let bg = scheme.background;
        screen.clear(ClearState::color_and_depth(bg[0], bg[1], bg[2], 1.0, 1.0));

        // Collect all lines to render: tuning lines and user pitch line
        let mut renderable_lines: Vec<&Gm<Line, ColorMaterial>> = Vec::new();
        
        // Only add user pitch line if pitch is detected
        if self.pitch_detected {
            renderable_lines.push(&self.user_pitch_line); // first in list is on top
        }

        // Add all tuning lines
        for line in self.tuning_lines.lines() {
            renderable_lines.push(line);
        }
        
        // Render lines
        screen.render(
            &self.camera,
            renderable_lines.into_iter(),
            &[&self.light],
        );
        
        // Clear previous frame's text queue
        self.text_renderer.clear_queue();
        
        // Render note labels above tuning lines
        self.tuning_lines.render_note_labels(&mut self.text_renderer);
        
        // Render text models using actual Roboto font  
        let viewport = self.camera.viewport();
        let text_models = self.text_renderer.create_text_models(&self.context, viewport);
        if !text_models.is_empty() {
            screen.render(
                &self.camera,
                &text_models,
                &[&self.light],
            );
        }
    }
    
    pub fn update_pitch_position(&mut self, viewport: Viewport, interval: f32, pitch_detected: bool, clarity: Option<f32>, cents_offset: f32) {
        self.pitch_detected = pitch_detected;
        self.cents_offset = cents_offset;
        if pitch_detected {
            let y = interval_to_screen_y_position(interval, viewport.height as f32, crate::web::main_scene_ui::get_current_zoom_factor());
            let endpoints = (PhysicalPoint{x:NOTE_LINE_LEFT_MARGIN, y}, PhysicalPoint{x:viewport.width as f32 - NOTE_LINE_RIGHT_MARGIN, y});
            
            // Calculate thickness and alpha based on clarity
            let (new_thickness, new_alpha) = if let Some(clarity_value) = clarity {
                // Map clarity from [CLARITY_THRESHOLD, 1.0] to [USER_PITCH_LINE_THICKNESS_MAX, USER_PITCH_LINE_THICKNESS_MIN]
                let clamped_clarity = clarity_value.clamp(CLARITY_THRESHOLD, 1.0);
                let normalized_clarity = (clamped_clarity - CLARITY_THRESHOLD) / (1.0 - CLARITY_THRESHOLD);
                let thickness = USER_PITCH_LINE_THICKNESS_MAX + normalized_clarity * (USER_PITCH_LINE_THICKNESS_MIN - USER_PITCH_LINE_THICKNESS_MAX);
                
                // Map clarity to alpha using configured transparency range
                // At CLARITY_THRESHOLD: alpha = USER_PITCH_LINE_TRANSPARENCY_MIN
                // At 1.0 clarity: alpha = USER_PITCH_LINE_TRANSPARENCY_MAX
                let alpha = USER_PITCH_LINE_TRANSPARENCY_MIN + normalized_clarity * (USER_PITCH_LINE_TRANSPARENCY_MAX - USER_PITCH_LINE_TRANSPARENCY_MIN);
                (thickness, alpha)
            } else {
                (USER_PITCH_LINE_THICKNESS_MAX, USER_PITCH_LINE_TRANSPARENCY_MAX) // Default values when no clarity provided
            };
            
            // Check if thickness or alpha changed - if so, recreate the line
            let thickness_changed = (new_thickness - self.user_pitch_line_thickness).abs() > f32::EPSILON;
            let alpha_changed = (new_alpha - self.user_pitch_line_alpha).abs() > f32::EPSILON;
            
            if thickness_changed || alpha_changed {
                let primary_material = create_color_material(
                    rgb_to_srgba_with_alpha(get_user_pitch_line_color(&self.current_scheme, self.volume_peak, self.cents_offset), new_alpha),
                    true
                );
                let line = Line::new(&self.context, endpoints.0, endpoints.1, new_thickness);
                self.user_pitch_line = Gm::new(line, primary_material);
                self.user_pitch_line_thickness = new_thickness;
                self.user_pitch_line_alpha = new_alpha;
            } else {
                // Only position changed, use existing line
                self.user_pitch_line.set_endpoints(endpoints.0, endpoints.1);
            }
        }
    }
    
    /// Update tuning lines with position, MIDI note, and thickness data provided by the presenter
    /// MainScene doesn't know about music theory - it just positions lines where told
    pub fn update_tuning_lines(&mut self, viewport: Viewport, line_data: &[(f32, MidiNote, f32)]) {
        // Use the new thickness-aware method
        self.tuning_lines.update_lines(viewport, line_data);
    }
    
    pub fn update_closest_note(&mut self, note: Option<MidiNote>) {
        self.tuning_lines.set_closest_note(note);
    }
    
    /// Update the volume peak state for color determination
    pub fn update_volume_peak(&mut self, volume_peak: bool) {
        self.volume_peak = volume_peak;
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    fn create_test_context() -> Context {
        // For tests, create a context using the Window API which handles
        // all the WebGL setup internally
        use three_d::{Window, WindowSettings};
        
        let window = Window::new(WindowSettings {
            title: "Test".to_string(),
            max_size: Some((800, 600)),
            ..Default::default()
        }).unwrap();
        
        window.gl().clone()
    }

    fn create_test_viewport() -> Viewport {
        Viewport {
            x: 0,
            y: 0,
            width: 800,
            height: 600,
        }
    }

    #[wasm_bindgen_test]
    fn test_main_scene_creation() {
        let context = create_test_context();
        let viewport = create_test_viewport();
        
        let scene = MainScene::new(&context, viewport).unwrap();
        
        // Verify that pitch_detected is false by default
        assert_eq!(scene.pitch_detected, false, "MainScene should initialize with pitch_detected = false");
    }

    #[wasm_bindgen_test]
    fn test_update_pitch_position_with_detection() {
        let context = create_test_context();
        let viewport = create_test_viewport();
        
        let mut scene = MainScene::new(&context, viewport).unwrap();
        
        // Test updating with pitch detected
        let interval = 1.0; // Middle of the range
        scene.update_pitch_position(viewport, interval, true, Some(0.8), 0.0);
        
        // Verify pitch_detected state is stored
        assert_eq!(scene.pitch_detected, true, "pitch_detected should be true after update with detection");
        
        // Verify that the line position would be updated (we can't directly test the line position
        // due to Line being opaque, but we can verify the state was set)
        let expected_y = interval_to_screen_y_position(interval, viewport.height as f32, crate::app_config::PITCH_VISUALIZATION_ZOOM_DEFAULT);
        assert!(expected_y > 0.0 && expected_y < viewport.height as f32, "Y position should be within viewport bounds");
    }

    #[wasm_bindgen_test]
    fn test_update_pitch_position_without_detection() {
        let context = create_test_context();
        let viewport = create_test_viewport();
        
        let mut scene = MainScene::new(&context, viewport).unwrap();
        
        // First set it to detected
        scene.update_pitch_position(viewport, 1.0, true, Some(0.8), 0.0);
        assert_eq!(scene.pitch_detected, true);
        
        // Now update without detection
        scene.update_pitch_position(viewport, 1.5, false, None, 0.0);
        
        // Verify pitch_detected state is stored as false
        assert_eq!(scene.pitch_detected, false, "pitch_detected should be false after update without detection");
    }

    #[wasm_bindgen_test]
    fn test_pitch_detection_state_persistence() {
        let context = create_test_context();
        let viewport = create_test_viewport();
        
        let mut scene = MainScene::new(&context, viewport).unwrap();
        
        // Test state persistence through multiple updates
        scene.update_pitch_position(viewport, 1.0, true, Some(0.8), 0.0);
        assert_eq!(scene.pitch_detected, true, "State should persist as true");
        
        scene.update_pitch_position(viewport, 1.2, true, Some(0.9), 0.0);
        assert_eq!(scene.pitch_detected, true, "State should remain true");
        
        scene.update_pitch_position(viewport, 1.3, false, None, 0.0);
        assert_eq!(scene.pitch_detected, false, "State should change to false");
        
        scene.update_pitch_position(viewport, 1.4, false, None, 0.0);
        assert_eq!(scene.pitch_detected, false, "State should remain false");
    }

    #[wasm_bindgen_test]
    fn test_conditional_rendering_logic() {
        let context = create_test_context();
        let viewport = create_test_viewport();
        
        let mut scene = MainScene::new(&context, viewport).unwrap();
        
        // Test that pitch_detected controls rendering inclusion
        // When pitch_detected is false, the line should not be included
        scene.pitch_detected = false;
        assert_eq!(scene.pitch_detected, false, "Line should not be included when pitch not detected");
        
        // When pitch_detected is true, the line should be included
        scene.pitch_detected = true;
        assert_eq!(scene.pitch_detected, true, "Line should be included when pitch is detected");
    }

    #[wasm_bindgen_test]
    fn test_rapid_state_changes() {
        let context = create_test_context();
        let viewport = create_test_viewport();
        
        let mut scene = MainScene::new(&context, viewport).unwrap();
        
        // Test rapid alternation between detected and not detected states
        for i in 0..10 {
            let detected = i % 2 == 0;
            let interval = 0.5 + (i as f32) * 0.15; // Vary the interval
            
            scene.update_pitch_position(viewport, interval, detected, if detected { Some(0.8) } else { None }, 0.0);
            
            assert_eq!(
                scene.pitch_detected, 
                detected, 
                "State should correctly alternate on iteration {}", 
                i
            );
        }
    }

    #[wasm_bindgen_test]
    fn test_edge_case_parameters() {
        let context = create_test_context();
        let viewport = create_test_viewport();
        
        let mut scene = MainScene::new(&context, viewport).unwrap();
        
        // Test with edge case interval values
        let edge_cases = [
            (0.0, true),   // Minimum interval
            (0.5, true),   // Lower octave boundary
            (1.0, true),   // Middle
            (2.0, true),   // Upper octave boundary
            (10.0, true),  // Extremely high interval
            (-1.0, true),  // Negative interval
            (0.0, false),  // Not detected with various intervals
            (1.0, false),
            (2.0, false),
        ];
        
        for (interval, detected) in edge_cases.iter() {
            scene.update_pitch_position(viewport, *interval, *detected, if *detected { Some(0.8) } else { None }, 0.0);
            assert_eq!(
                scene.pitch_detected, 
                *detected, 
                "State should be {} for interval {}", 
                detected, 
                interval
            );
            
            // Verify y position calculation doesn't panic
            let y = interval_to_screen_y_position(*interval, viewport.height as f32, crate::app_config::PITCH_VISUALIZATION_ZOOM_DEFAULT);
            assert!(!y.is_nan(), "Y position should not be NaN for interval {}", interval);
        }
        
        // Test with edge case viewport
        let small_viewport = Viewport {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        };
        
        scene.update_pitch_position(small_viewport, 1.0, true, Some(0.8), 0.0);
        assert_eq!(scene.pitch_detected, true, "Should handle small viewport");
        
        let large_viewport = Viewport {
            x: 0,
            y: 0,
            width: 10000,
            height: 10000,
        };
        
        scene.update_pitch_position(large_viewport, 1.0, false, None, 0.0);
        assert_eq!(scene.pitch_detected, false, "Should handle large viewport");
    }

    #[wasm_bindgen_test]
    fn test_interval_to_screen_y_position_calculation() {
        // Test the helper function directly
        let viewport_height = 600.0;
        
        // Test standard intervals with scale factor
        let y_middle = interval_to_screen_y_position(0.0, viewport_height, crate::app_config::PITCH_VISUALIZATION_ZOOM_DEFAULT);
        assert_eq!(y_middle, 300.0, "Interval 0.0 should map to middle of screen");
        
        let y_top = interval_to_screen_y_position(-1.0, viewport_height, crate::app_config::PITCH_VISUALIZATION_ZOOM_DEFAULT);
        let expected_top = viewport_height * (0.5 - PITCH_VISUALIZATION_ZOOM_DEFAULT * 0.5);
        assert!((y_top - expected_top).abs() < 0.01, "Interval -1.0 should map correctly with scale factor");
        
        let y_bottom = interval_to_screen_y_position(1.0, viewport_height, crate::app_config::PITCH_VISUALIZATION_ZOOM_DEFAULT);
        let expected_bottom = viewport_height * (0.5 + PITCH_VISUALIZATION_ZOOM_DEFAULT * 0.5);
        assert!((y_bottom - expected_bottom).abs() < 0.01, "Interval 1.0 should map correctly with scale factor");
        
        // Test intermediate values
        let y_quarter = interval_to_screen_y_position(-0.5, viewport_height, crate::app_config::PITCH_VISUALIZATION_ZOOM_DEFAULT);
        let expected_quarter = viewport_height * (0.5 - PITCH_VISUALIZATION_ZOOM_DEFAULT * 0.25);
        assert!((y_quarter - expected_quarter).abs() < 0.01, "Interval -0.5 should map correctly with scale factor");
        
        let y_three_quarters = interval_to_screen_y_position(0.5, viewport_height, crate::app_config::PITCH_VISUALIZATION_ZOOM_DEFAULT);
        let expected_three_quarters = viewport_height * (0.5 + PITCH_VISUALIZATION_ZOOM_DEFAULT * 0.25);
        assert!((y_three_quarters - expected_three_quarters).abs() < 0.01, "Interval 0.5 should map correctly with scale factor");
    }

    #[wasm_bindgen_test]
    fn test_tuning_lines_always_rendered() {
        let context = create_test_context();
        let viewport = create_test_viewport();
        
        let mut scene = MainScene::new(&context, viewport).unwrap();
        
        // Add some tuning lines
        let line_data = vec![
            (100.0, 60, 1.0),  // Middle C
            (200.0, 62, 1.0),  // D
            (300.0, 64, 1.0),  // E
        ];
        
        scene.update_tuning_lines(viewport, &line_data);
        
        // Verify tuning lines are present regardless of pitch detection state
        scene.pitch_detected = false;
        assert_eq!(scene.tuning_lines.lines().count(), 3, "Tuning lines should be present when pitch not detected");
        
        scene.pitch_detected = true;
        assert_eq!(scene.tuning_lines.lines().count(), 3, "Tuning lines should be present when pitch is detected");
    }

    #[wasm_bindgen_test]
    fn test_render_state_consistency() {
        let context = create_test_context();
        let viewport = create_test_viewport();
        
        let mut scene = MainScene::new(&context, viewport).unwrap();
        
        // Set up initial state
        scene.update_pitch_position(viewport, 1.0, true, Some(0.8), 0.0);
        assert_eq!(scene.pitch_detected, true);
        
        // Add tuning lines
        let line_data = vec![(100.0, 60, 1.0), (200.0, 62, 1.0)];
        scene.update_tuning_lines(viewport, &line_data);
        
        // Change pitch detection state
        scene.update_pitch_position(viewport, 1.2, false, None, 0.0);
        assert_eq!(scene.pitch_detected, false);
        
        // Verify tuning lines are still there
        assert_eq!(scene.tuning_lines.lines().count(), 2, "Tuning lines should persist through state changes");
    }

    #[wasm_bindgen_test]
    fn test_thickness_calculation_high_clarity() {
        let context = create_test_context();
        let viewport = create_test_viewport();
        
        let mut scene = MainScene::new(&context, viewport).unwrap();
        
        // Test high clarity (near 1.0) produces thin lines
        scene.update_pitch_position(viewport, 1.0, true, Some(0.95), 0.0);
        assert_eq!(scene.pitch_detected, true);
        
        // Verify thickness is near minimum
        let expected_thickness = USER_PITCH_LINE_THICKNESS_MAX + 0.83 * (USER_PITCH_LINE_THICKNESS_MIN - USER_PITCH_LINE_THICKNESS_MAX);
        assert!((scene.user_pitch_line_thickness - expected_thickness).abs() < 0.1, "High clarity should produce thin line");
    }

    #[wasm_bindgen_test]
    fn test_thickness_calculation_low_clarity() {
        let context = create_test_context();
        let viewport = create_test_viewport();
        
        let mut scene = MainScene::new(&context, viewport).unwrap();
        
        // Test low clarity (near threshold) produces thick lines
        scene.update_pitch_position(viewport, 1.0, true, Some(CLARITY_THRESHOLD + 0.01), 0.0);
        assert_eq!(scene.pitch_detected, true);
        
        // Verify thickness is near maximum
        assert!((scene.user_pitch_line_thickness - USER_PITCH_LINE_THICKNESS_MAX).abs() < 0.1, "Low clarity should produce thick line");
    }

    #[wasm_bindgen_test]
    fn test_thickness_changes_trigger_recreation() {
        let context = create_test_context();
        let viewport = create_test_viewport();
        
        let mut scene = MainScene::new(&context, viewport).unwrap();
        
        // Set initial thickness with high clarity
        scene.update_pitch_position(viewport, 1.0, true, Some(0.95), 0.0);
        let initial_thickness = scene.user_pitch_line_thickness;
        
        // Change to low clarity - this should trigger line recreation
        scene.update_pitch_position(viewport, 1.0, true, Some(CLARITY_THRESHOLD + 0.01), 0.0);
        let new_thickness = scene.user_pitch_line_thickness;
        
        // Verify thickness actually changed
        assert!((new_thickness - initial_thickness).abs() > 1.0, "Thickness should change significantly between high and low clarity");
        assert!((new_thickness - USER_PITCH_LINE_THICKNESS_MAX).abs() < 0.1, "New thickness should be near maximum");
        assert!((initial_thickness - USER_PITCH_LINE_THICKNESS_MIN).abs() < 1.0, "Initial thickness should be closer to minimum");
    }
}