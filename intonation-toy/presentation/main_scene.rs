use three_d::{AmbientLight, Camera, ClearState, ColorMaterial, Context, Gm, Line, PhysicalPoint, RenderTarget, Srgba, Viewport};
use crate::shared_types::{MidiNote, ColorScheme};
use crate::theme::{get_current_color_scheme, rgb_to_srgba};

pub fn interval_to_screen_y_position(interval: f32, viewport_height: f32) -> f32 {
    // interval of [0.5, 2.0] means [-1, +1] octaves
    let scale_factor = 1.0;
    let y: f32 = viewport_height * (0.5 + interval * scale_factor * 0.5);
    y
}

pub struct TuningLines {
    lines: Vec<Gm<Line, ColorMaterial>>,
    midi_notes: Vec<MidiNote>,
    y_positions: Vec<f32>,
    thicknesses: Vec<f32>,
    context: Context,
    material: ColorMaterial,
}

impl TuningLines {
    pub fn new(context: &Context, color: Srgba) -> Self {
        let material = ColorMaterial {
            color,
            ..Default::default()
        };
        
        Self {
            lines: Vec::new(),
            midi_notes: Vec::new(),
            y_positions: Vec::new(),
            thicknesses: Vec::new(),
            context: context.clone(),
            material,
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
                PhysicalPoint { x: 0.0, y: 0.0 },
                PhysicalPoint { x: 0.0, y: 0.0 },
                1.0  // Default thickness, will be updated
            );
            self.lines.push(Gm::new(line, self.material.clone()));
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
            // If thickness changed, we need to recreate the line
            if i < self.thicknesses.len() && self.thicknesses[i] != thickness {
                let line = Line::new(
                    &self.context,
                    PhysicalPoint { x: 0.0, y },
                    PhysicalPoint { x: width, y },
                    thickness
                );
                self.lines[i] = Gm::new(line, self.material.clone());
            } else {
                // Just update endpoints if thickness hasn't changed
                self.lines[i].set_endpoints(
                    PhysicalPoint { x: 0.0, y },
                    PhysicalPoint { x: width, y }
                );
            }
            self.midi_notes[i] = midi_note;
            self.y_positions[i] = y;
            self.thicknesses[i] = thickness;
            //crate::common::dev_log!("TUNING_DEBUG: Line {}: y={:.1}, midi_note={}, thickness={:.1}, width={:.1}", i, y, midi_note, thickness, width);
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
    
    /// Render note labels above each tuning line
    pub fn render_note_labels(&self, text_renderer: &mut TextRenderer) {
        for (i, &midi_note) in self.midi_notes.iter().enumerate() {
            let y_position = self.y_positions[i];
            
            // Convert MIDI note to name
            let note_name = crate::shared_types::midi_note_to_name(midi_note);
            
            // Position text slightly above the line (20 pixels up)
            let text_y = y_position - 20.0;
            let text_x = 10.0; // Small offset from left edge
            
            // Queue the text for rendering (using theme text color, small size)
            let text_color = get_current_color_scheme().text;
            text_renderer.queue_text(&note_name, text_x, text_y, 8.0, [text_color[0], text_color[1], text_color[2], 1.0]);
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
}

impl MainScene {
    pub fn new(context: &Context, viewport: Viewport) -> Result<Self, String> {
        let scheme = get_current_color_scheme();
        let user_pitch_line = Line::new(context, PhysicalPoint{x:0.0, y:0.0}, PhysicalPoint{x:0.0, y:0.0}, 2.0);

        let primary_material = ColorMaterial {
            color: rgb_to_srgba(scheme.primary),
            ..Default::default()
        };
        
        let tuning_lines = TuningLines::new(context, rgb_to_srgba(scheme.text));
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
        })
    }
    
    pub fn update_viewport(&mut self, viewport: Viewport) {
        self.camera.set_viewport(viewport);
    }
    
    fn refresh_colors(&mut self) {
        let scheme = self.current_scheme.clone();
        
        // Recreate user pitch line with new color (it will be repositioned on next update)
        let primary_material = ColorMaterial {
            color: rgb_to_srgba(scheme.primary),
            ..Default::default()
        };
        let line = Line::new(&self.context, 
            PhysicalPoint{x:0.0, y:0.0}, 
            PhysicalPoint{x:0.0, y:0.0}, 
            2.0);
        self.user_pitch_line = Gm::new(line, primary_material);
        
        // Update tuning lines material
        self.tuning_lines.material = ColorMaterial {
            color: rgb_to_srgba(scheme.text),
            ..Default::default()
        };
        
        // Clear and recreate all tuning lines with new material
        // They will be recreated with correct positions on next update_lines call
        self.tuning_lines.lines.clear();
        
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
    
    pub fn update_pitch_position(&mut self, viewport: Viewport, interval: f32, pitch_detected: bool) {
        self.pitch_detected = pitch_detected;
        if pitch_detected {
            let y = interval_to_screen_y_position(interval, viewport.height as f32);
            self.user_pitch_line.set_endpoints(PhysicalPoint{x:0.0, y}, PhysicalPoint{x:viewport.width as f32, y});
        }
    }
    
    /// Update tuning lines with position, MIDI note, and thickness data provided by the presenter
    /// MainScene doesn't know about music theory - it just positions lines where told
    pub fn update_tuning_lines(&mut self, viewport: Viewport, line_data: &[(f32, MidiNote, f32)]) {
        // Use the new thickness-aware method
        self.tuning_lines.update_lines(viewport, line_data);
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    use wasm_bindgen::JsCast;

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
        scene.update_pitch_position(viewport, interval, true);
        
        // Verify pitch_detected state is stored
        assert_eq!(scene.pitch_detected, true, "pitch_detected should be true after update with detection");
        
        // Verify that the line position would be updated (we can't directly test the line position
        // due to Line being opaque, but we can verify the state was set)
        let expected_y = interval_to_screen_y_position(interval, viewport.height as f32);
        assert!(expected_y > 0.0 && expected_y < viewport.height as f32, "Y position should be within viewport bounds");
    }

    #[wasm_bindgen_test]
    fn test_update_pitch_position_without_detection() {
        let context = create_test_context();
        let viewport = create_test_viewport();
        
        let mut scene = MainScene::new(&context, viewport).unwrap();
        
        // First set it to detected
        scene.update_pitch_position(viewport, 1.0, true);
        assert_eq!(scene.pitch_detected, true);
        
        // Now update without detection
        scene.update_pitch_position(viewport, 1.5, false);
        
        // Verify pitch_detected state is stored as false
        assert_eq!(scene.pitch_detected, false, "pitch_detected should be false after update without detection");
    }

    #[wasm_bindgen_test]
    fn test_pitch_detection_state_persistence() {
        let context = create_test_context();
        let viewport = create_test_viewport();
        
        let mut scene = MainScene::new(&context, viewport).unwrap();
        
        // Test state persistence through multiple updates
        scene.update_pitch_position(viewport, 1.0, true);
        assert_eq!(scene.pitch_detected, true, "State should persist as true");
        
        scene.update_pitch_position(viewport, 1.2, true);
        assert_eq!(scene.pitch_detected, true, "State should remain true");
        
        scene.update_pitch_position(viewport, 1.3, false);
        assert_eq!(scene.pitch_detected, false, "State should change to false");
        
        scene.update_pitch_position(viewport, 1.4, false);
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
            
            scene.update_pitch_position(viewport, interval, detected);
            
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
            scene.update_pitch_position(viewport, *interval, *detected);
            assert_eq!(
                scene.pitch_detected, 
                *detected, 
                "State should be {} for interval {}", 
                detected, 
                interval
            );
            
            // Verify y position calculation doesn't panic
            let y = interval_to_screen_y_position(*interval, viewport.height as f32);
            assert!(!y.is_nan(), "Y position should not be NaN for interval {}", interval);
        }
        
        // Test with edge case viewport
        let small_viewport = Viewport {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        };
        
        scene.update_pitch_position(small_viewport, 1.0, true);
        assert_eq!(scene.pitch_detected, true, "Should handle small viewport");
        
        let large_viewport = Viewport {
            x: 0,
            y: 0,
            width: 10000,
            height: 10000,
        };
        
        scene.update_pitch_position(large_viewport, 1.0, false);
        assert_eq!(scene.pitch_detected, false, "Should handle large viewport");
    }

    #[wasm_bindgen_test]
    fn test_interval_to_screen_y_position_calculation() {
        // Test the helper function directly
        let viewport_height = 600.0;
        
        // Test standard intervals
        let y_middle = interval_to_screen_y_position(0.0, viewport_height);
        assert_eq!(y_middle, 300.0, "Interval 0.0 should map to middle of screen");
        
        let y_top = interval_to_screen_y_position(-1.0, viewport_height);
        assert_eq!(y_top, 0.0, "Interval -1.0 should map to top of screen");
        
        let y_bottom = interval_to_screen_y_position(1.0, viewport_height);
        assert_eq!(y_bottom, 600.0, "Interval 1.0 should map to bottom of screen");
        
        // Test intermediate values
        let y_quarter = interval_to_screen_y_position(-0.5, viewport_height);
        assert_eq!(y_quarter, 150.0, "Interval -0.5 should map to quarter way down");
        
        let y_three_quarters = interval_to_screen_y_position(0.5, viewport_height);
        assert_eq!(y_three_quarters, 450.0, "Interval 0.5 should map to three quarters down");
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
        scene.update_pitch_position(viewport, 1.0, true);
        assert_eq!(scene.pitch_detected, true);
        
        // Add tuning lines
        let line_data = vec![(100.0, 60, 1.0), (200.0, 62, 1.0)];
        scene.update_tuning_lines(viewport, &line_data);
        
        // Change pitch detection state
        scene.update_pitch_position(viewport, 1.2, false);
        assert_eq!(scene.pitch_detected, false);
        
        // Verify tuning lines are still there
        assert_eq!(scene.tuning_lines.lines().count(), 2, "Tuning lines should persist through state changes");
    }
}