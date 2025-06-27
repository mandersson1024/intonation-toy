use super::{GraphicsError, RenderCommand, Vertex, PrimitiveTopology};

/// Trait for audio-specific visualization rendering
pub trait VisualizationRenderer {
    /// Initialize the visualization renderer
    fn initialize(&mut self, config: &VisualizationConfig) -> Result<(), GraphicsError>;
    
    /// Render pitch visualization data
    fn render_pitch_visualization(&mut self, data: &PitchVisualizationData) -> Result<Vec<RenderCommand>, GraphicsError>;
    
    /// Render waveform visualization
    fn render_waveform(&mut self, data: &WaveformData) -> Result<Vec<RenderCommand>, GraphicsError>;
    
    /// Render frequency spectrum visualization
    fn render_spectrum(&mut self, data: &SpectrumData) -> Result<Vec<RenderCommand>, GraphicsError>;
    
    /// Update visualization with new audio data
    fn update_visualization(&mut self, audio_data: &AudioVisualizationData) -> Result<(), GraphicsError>;
    
    /// Set visualization parameters
    fn set_visualization_params(&mut self, params: &VisualizationParams) -> Result<(), GraphicsError>;
    
    /// Get current visualization state
    fn get_visualization_state(&self) -> VisualizationState;
    
    /// Cleanup visualization resources
    fn cleanup(&mut self) -> Result<(), GraphicsError>;
}

/// Configuration for visualization renderer
#[derive(Debug, Clone)]
pub struct VisualizationConfig {
    pub enable_pitch_visualization: bool,
    pub enable_waveform: bool,
    pub enable_spectrum: bool,
    pub target_fps: u32,
    pub max_history_samples: usize,
    pub color_scheme: ColorScheme,
}

/// Color scheme for visualizations
#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub background: [f32; 4],
    pub primary: [f32; 4],
    pub secondary: [f32; 4],
    pub accent: [f32; 4],
    pub grid: [f32; 4],
}

/// Pitch visualization data
#[derive(Debug, Clone)]
pub struct PitchVisualizationData {
    pub current_pitch: Option<f32>,
    pub pitch_history: Vec<f32>,
    pub confidence: f32,
    pub target_pitch: Option<f32>,
    pub pitch_deviation: f32,
}

/// Waveform visualization data
#[derive(Debug, Clone)]
pub struct WaveformData {
    pub samples: Vec<f32>,
    pub sample_rate: f32,
    pub time_window_ms: f32,
}

/// Frequency spectrum visualization data
#[derive(Debug, Clone)]
pub struct SpectrumData {
    pub frequencies: Vec<f32>,
    pub magnitudes: Vec<f32>,
    pub sample_rate: f32,
    pub fft_size: usize,
}

/// Combined audio visualization data
#[derive(Debug, Clone)]
pub struct AudioVisualizationData {
    pub pitch: Option<PitchVisualizationData>,
    pub waveform: Option<WaveformData>,
    pub spectrum: Option<SpectrumData>,
    pub timestamp: f64,
}

/// Visualization rendering parameters
#[derive(Debug, Clone)]
pub struct VisualizationParams {
    pub scale: f32,
    pub opacity: f32,
    pub animation_speed: f32,
    pub show_grid: bool,
    pub show_labels: bool,
    pub pitch_range: (f32, f32),
    pub frequency_range: (f32, f32),
}

/// Current state of visualization renderer
#[derive(Debug, Clone)]
pub struct VisualizationState {
    pub initialized: bool,
    pub active_visualizations: Vec<VisualizationType>,
    pub frame_count: u64,
    pub last_update_time: f64,
    pub performance_metrics: VisualizationMetrics,
}

/// Types of available visualizations
#[derive(Debug, Clone, PartialEq)]
pub enum VisualizationType {
    Pitch,
    Waveform,
    Spectrum,
}

/// Performance metrics for visualization rendering
#[derive(Debug, Clone)]
pub struct VisualizationMetrics {
    pub render_time_ms: f32,
    pub vertices_generated: u32,
    pub commands_generated: u32,
    pub fps: f32,
}

/// Implementation of visualization renderer for audio graphics
pub struct AudioVisualizationRenderer {
    config: Option<VisualizationConfig>,
    state: VisualizationState,
    pitch_history: Vec<f32>,
    waveform_buffer: Vec<f32>,
    spectrum_buffer: Vec<f32>,
    initialized: bool,
}

impl AudioVisualizationRenderer {
    /// Create a new audio visualization renderer
    pub fn new() -> Self {
        Self {
            config: None,
            state: VisualizationState::default(),
            pitch_history: Vec::new(),
            waveform_buffer: Vec::new(),
            spectrum_buffer: Vec::new(),
            initialized: false,
        }
    }
    
    /// Generate vertices for pitch visualization
    fn generate_pitch_vertices(&self, data: &PitchVisualizationData) -> Result<Vec<Vertex>, GraphicsError> {
        let mut vertices = Vec::new();
        let config = self.config.as_ref().unwrap();
        
        // Generate pitch line visualization
        if let Some(current_pitch) = data.current_pitch {
            let normalized_pitch = self.normalize_pitch(current_pitch);
            let color = self.get_pitch_color(data.confidence);
            
            // Create a simple line representing current pitch
            vertices.push(Vertex {
                position: [-1.0, normalized_pitch, 0.0],
                color,
                texture_coords: [0.0, 0.0],
            });
            vertices.push(Vertex {
                position: [1.0, normalized_pitch, 0.0],
                color,
                texture_coords: [1.0, 0.0],
            });
        }
        
        // Generate pitch history visualization
        let history_len = data.pitch_history.len();
        if history_len > 1 {
            for (i, &pitch) in data.pitch_history.iter().enumerate() {
                let x = -1.0 + 2.0 * (i as f32 / history_len as f32);
                let y = self.normalize_pitch(pitch);
                let alpha = 0.3 + 0.7 * (i as f32 / history_len as f32);
                
                vertices.push(Vertex {
                    position: [x, y, 0.0],
                    color: [config.color_scheme.secondary[0], config.color_scheme.secondary[1], 
                           config.color_scheme.secondary[2], alpha],
                    texture_coords: [x * 0.5 + 0.5, y * 0.5 + 0.5],
                });
            }
        }
        
        Ok(vertices)
    }
    
    /// Generate vertices for waveform visualization
    fn generate_waveform_vertices(&self, data: &WaveformData) -> Result<Vec<Vertex>, GraphicsError> {
        let mut vertices = Vec::new();
        let config = self.config.as_ref().unwrap();
        
        let sample_count = data.samples.len();
        if sample_count < 2 {
            return Ok(vertices);
        }
        
        for (i, &sample) in data.samples.iter().enumerate() {
            let x = -1.0 + 2.0 * (i as f32 / sample_count as f32);
            let y = sample.clamp(-1.0, 1.0);
            
            vertices.push(Vertex {
                position: [x, y, 0.0],
                color: config.color_scheme.primary,
                texture_coords: [x * 0.5 + 0.5, y * 0.5 + 0.5],
            });
        }
        
        Ok(vertices)
    }
    
    /// Generate vertices for spectrum visualization
    fn generate_spectrum_vertices(&self, data: &SpectrumData) -> Result<Vec<Vertex>, GraphicsError> {
        let mut vertices = Vec::new();
        let config = self.config.as_ref().unwrap();
        
        let bin_count = data.magnitudes.len();
        if bin_count < 2 {
            return Ok(vertices);
        }
        
        for (i, &magnitude) in data.magnitudes.iter().enumerate() {
            let x = -1.0 + 2.0 * (i as f32 / bin_count as f32);
            let y = -1.0 + 2.0 * magnitude.clamp(0.0, 1.0);
            
            // Create a bar for each frequency bin
            let bar_width = 1.8 / bin_count as f32;
            let left = x - bar_width * 0.4;
            let right = x + bar_width * 0.4;
            
            // Bottom-left vertex
            vertices.push(Vertex {
                position: [left, -1.0, 0.0],
                color: config.color_scheme.accent,
                texture_coords: [0.0, 0.0],
            });
            
            // Bottom-right vertex
            vertices.push(Vertex {
                position: [right, -1.0, 0.0],
                color: config.color_scheme.accent,
                texture_coords: [1.0, 0.0],
            });
            
            // Top-right vertex
            vertices.push(Vertex {
                position: [right, y, 0.0],
                color: config.color_scheme.accent,
                texture_coords: [1.0, 1.0],
            });
            
            // Top-left vertex
            vertices.push(Vertex {
                position: [left, y, 0.0],
                color: config.color_scheme.accent,
                texture_coords: [0.0, 1.0],
            });
        }
        
        Ok(vertices)
    }
    
    /// Normalize pitch value to rendering coordinates
    fn normalize_pitch(&self, pitch: f32) -> f32 {
        // Simple logarithmic mapping for pitch visualization
        let log_pitch = (pitch / 440.0).log2();
        log_pitch.clamp(-2.0, 2.0) / 2.0
    }
    
    /// Get color for pitch based on confidence
    fn get_pitch_color(&self, confidence: f32) -> [f32; 4] {
        let config = self.config.as_ref().unwrap();
        let alpha = 0.5 + 0.5 * confidence;
        [config.color_scheme.primary[0], config.color_scheme.primary[1], 
         config.color_scheme.primary[2], alpha]
    }
}

impl VisualizationRenderer for AudioVisualizationRenderer {
    fn initialize(&mut self, config: &VisualizationConfig) -> Result<(), GraphicsError> {
        if self.initialized {
            return Ok(());
        }
        
        // Validate configuration
        if config.max_history_samples == 0 {
            return Err(GraphicsError::InvalidConfiguration(
                "Max history samples must be greater than 0".to_string()
            ));
        }
        
        if config.target_fps == 0 {
            return Err(GraphicsError::InvalidConfiguration(
                "Target FPS must be greater than 0".to_string()
            ));
        }
        
        self.config = Some(config.clone());
        
        // Initialize buffers
        self.pitch_history.reserve(config.max_history_samples);
        self.waveform_buffer.reserve(2048); // Common audio buffer size
        self.spectrum_buffer.reserve(1024); // Common FFT size
        
        self.state.initialized = true;
        self.initialized = true;
        
        Ok(())
    }
    
    fn render_pitch_visualization(&mut self, data: &PitchVisualizationData) -> Result<Vec<RenderCommand>, GraphicsError> {
        if !self.initialized {
            return Err(GraphicsError::ContextInitializationFailed(
                "Visualization renderer not initialized".to_string()
            ));
        }
        
        let vertices = self.generate_pitch_vertices(data)?;
        if vertices.is_empty() {
            return Ok(vec![]);
        }
        
        let commands = vec![
            RenderCommand::DrawVertices {
                vertices,
                topology: PrimitiveTopology::LineStrip,
            }
        ];
        
        self.state.performance_metrics.commands_generated += commands.len() as u32;
        
        Ok(commands)
    }
    
    fn render_waveform(&mut self, data: &WaveformData) -> Result<Vec<RenderCommand>, GraphicsError> {
        if !self.initialized {
            return Err(GraphicsError::ContextInitializationFailed(
                "Visualization renderer not initialized".to_string()
            ));
        }
        
        let vertices = self.generate_waveform_vertices(data)?;
        if vertices.is_empty() {
            return Ok(vec![]);
        }
        
        let commands = vec![
            RenderCommand::DrawVertices {
                vertices,
                topology: PrimitiveTopology::LineStrip,
            }
        ];
        
        self.state.performance_metrics.commands_generated += commands.len() as u32;
        
        Ok(commands)
    }
    
    fn render_spectrum(&mut self, data: &SpectrumData) -> Result<Vec<RenderCommand>, GraphicsError> {
        if !self.initialized {
            return Err(GraphicsError::ContextInitializationFailed(
                "Visualization renderer not initialized".to_string()
            ));
        }
        
        let vertices = self.generate_spectrum_vertices(data)?;
        if vertices.is_empty() {
            return Ok(vec![]);
        }
        
        // Generate indices for triangle list rendering (two triangles per bar)
        let mut indices = Vec::new();
        let quad_count = vertices.len() / 4;
        
        for i in 0..quad_count {
            let base = (i * 4) as u32;
            // First triangle
            indices.extend_from_slice(&[base, base + 1, base + 2]);
            // Second triangle
            indices.extend_from_slice(&[base, base + 2, base + 3]);
        }
        
        let commands = vec![
            RenderCommand::DrawIndexed {
                vertices,
                indices,
                topology: PrimitiveTopology::TriangleList,
            }
        ];
        
        self.state.performance_metrics.commands_generated += commands.len() as u32;
        
        Ok(commands)
    }
    
    fn update_visualization(&mut self, audio_data: &AudioVisualizationData) -> Result<(), GraphicsError> {
        if !self.initialized {
            return Err(GraphicsError::ContextInitializationFailed(
                "Visualization renderer not initialized".to_string()
            ));
        }
        
        let config = self.config.as_ref().unwrap();
        
        // Update pitch history
        if let Some(pitch_data) = &audio_data.pitch {
            if let Some(current_pitch) = pitch_data.current_pitch {
                self.pitch_history.push(current_pitch);
                if self.pitch_history.len() > config.max_history_samples {
                    self.pitch_history.remove(0);
                }
            }
        }
        
        // Update waveform buffer
        if let Some(waveform_data) = &audio_data.waveform {
            self.waveform_buffer = waveform_data.samples.clone();
        }
        
        // Update spectrum buffer
        if let Some(spectrum_data) = &audio_data.spectrum {
            self.spectrum_buffer = spectrum_data.magnitudes.clone();
        }
        
        self.state.last_update_time = audio_data.timestamp;
        self.state.frame_count += 1;
        
        Ok(())
    }
    
    fn set_visualization_params(&mut self, params: &VisualizationParams) -> Result<(), GraphicsError> {
        // TODO: Apply visualization parameters
        // For now, just validate the parameters
        if params.scale <= 0.0 {
            return Err(GraphicsError::InvalidConfiguration(
                "Scale must be greater than 0".to_string()
            ));
        }
        
        if params.opacity < 0.0 || params.opacity > 1.0 {
            return Err(GraphicsError::InvalidConfiguration(
                "Opacity must be between 0 and 1".to_string()
            ));
        }
        
        Ok(())
    }
    
    fn get_visualization_state(&self) -> VisualizationState {
        self.state.clone()
    }
    
    fn cleanup(&mut self) -> Result<(), GraphicsError> {
        self.pitch_history.clear();
        self.waveform_buffer.clear();
        self.spectrum_buffer.clear();
        self.config = None;
        self.state = VisualizationState::default();
        self.initialized = false;
        
        Ok(())
    }
}

impl Default for AudioVisualizationRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            enable_pitch_visualization: true,
            enable_waveform: true,
            enable_spectrum: true,
            target_fps: 60,
            max_history_samples: 1000,
            color_scheme: ColorScheme::default(),
        }
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            background: [0.1, 0.1, 0.1, 1.0],
            primary: [0.0, 0.8, 1.0, 1.0],
            secondary: [0.8, 0.8, 0.8, 0.5],
            accent: [1.0, 0.6, 0.0, 0.8],
            grid: [0.3, 0.3, 0.3, 0.3],
        }
    }
}

impl Default for VisualizationState {
    fn default() -> Self {
        Self {
            initialized: false,
            active_visualizations: Vec::new(),
            frame_count: 0,
            last_update_time: 0.0,
            performance_metrics: VisualizationMetrics::default(),
        }
    }
}

impl Default for VisualizationMetrics {
    fn default() -> Self {
        Self {
            render_time_ms: 0.0,
            vertices_generated: 0,
            commands_generated: 0,
            fps: 0.0,
        }
    }
}

impl Default for VisualizationParams {
    fn default() -> Self {
        Self {
            scale: 1.0,
            opacity: 1.0,
            animation_speed: 1.0,
            show_grid: true,
            show_labels: true,
            pitch_range: (80.0, 1000.0),
            frequency_range: (20.0, 20000.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_visualization_renderer_creation() {
        let renderer = AudioVisualizationRenderer::new();
        assert!(!renderer.initialized);
        assert!(!renderer.state.initialized);
    }
    
    #[test]
    fn test_visualization_renderer_initialization() {
        let mut renderer = AudioVisualizationRenderer::new();
        let config = VisualizationConfig::default();
        
        let result = renderer.initialize(&config);
        assert!(result.is_ok());
        assert!(renderer.initialized);
        assert!(renderer.state.initialized);
    }
    
    #[test]
    fn test_pitch_visualization_generation() {
        let mut renderer = AudioVisualizationRenderer::new();
        let config = VisualizationConfig::default();
        renderer.initialize(&config).unwrap();
        
        let pitch_data = PitchVisualizationData {
            current_pitch: Some(440.0),
            pitch_history: vec![440.0, 441.0, 439.0],
            confidence: 0.8,
            target_pitch: Some(440.0),
            pitch_deviation: 1.0,
        };
        
        let result = renderer.render_pitch_visualization(&pitch_data);
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert!(!commands.is_empty());
    }
    
    #[test]
    fn test_color_scheme_default() {
        let scheme = ColorScheme::default();
        assert_eq!(scheme.background, [0.1, 0.1, 0.1, 1.0]);
        assert_eq!(scheme.primary, [0.0, 0.8, 1.0, 1.0]);
    }
}