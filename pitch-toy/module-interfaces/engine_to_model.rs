use observable_data::{DataSource, DataObserver, DataSourceSetter};

#[derive(Debug, Clone, PartialEq)]
pub struct Volume {
    pub peak: f32,
    pub rms: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pitch {
    Detected(f32, f32), // frequency, clarity
    NotDetected,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioAnalysis {
    pub volume_level: Volume,
    pub pitch: Pitch,
    pub fft_data: Option<Vec<f32>>, // roadmap
    pub timestamp: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudioError {
    MicrophonePermissionDenied,
    MicrophoneNotAvailable,
    ProcessingError(String),
    BrowserApiNotSupported,
    AudioContextInitFailed,
    AudioContextSuspended,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PermissionState {
    NotRequested,
    Requested,
    Granted,
    Denied,
}

pub struct EngineToModelInterface {
    audio_analysis_source: DataSource<Option<AudioAnalysis>>,
    audio_errors_source: DataSource<Vec<AudioError>>,
    permission_state_source: DataSource<PermissionState>,
}

impl EngineToModelInterface {
    pub fn new() -> Self {
        Self {
            audio_analysis_source: DataSource::new(None),
            audio_errors_source: DataSource::new(Vec::new()),
            permission_state_source: DataSource::new(PermissionState::NotRequested),
        }
    }

    /// Get a setter for the audio analysis data that the engine can use to push data
    pub fn audio_analysis_setter(&self) -> DataSourceSetter<Option<AudioAnalysis>> {
        self.audio_analysis_source.setter()
    }

    /// Get a setter for the audio errors that the engine can use to push errors
    pub fn audio_errors_setter(&self) -> DataSourceSetter<Vec<AudioError>> {
        self.audio_errors_source.setter()
    }

    /// Get a setter for the permission state that the engine can use to update permissions
    pub fn permission_state_setter(&self) -> DataSourceSetter<PermissionState> {
        self.permission_state_source.setter()
    }

    /// Get an observer for the audio analysis data that the model can use to read data
    pub fn audio_analysis_observer(&self) -> DataObserver<Option<AudioAnalysis>> {
        self.audio_analysis_source.observer()
    }

    /// Get an observer for the audio errors that the model can use to read errors
    pub fn audio_errors_observer(&self) -> DataObserver<Vec<AudioError>> {
        self.audio_errors_source.observer()
    }

    /// Get an observer for the permission state that the model can use to read permissions
    pub fn permission_state_observer(&self) -> DataObserver<PermissionState> {
        self.permission_state_source.observer()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    use observable_data::DataSetter;

    #[wasm_bindgen_test]
    fn test_interface_factory_system() {
        let interface = EngineToModelInterface::new();
        
        // Test that setters can be extracted
        let audio_setter = interface.audio_analysis_setter();
        let errors_setter = interface.audio_errors_setter();
        let permission_setter = interface.permission_state_setter();
        
        // Test that observers can be extracted
        let audio_observer = interface.audio_analysis_observer();
        let errors_observer = interface.audio_errors_observer();
        let permission_observer = interface.permission_state_observer();
        
        // Test that setters and observers work together
        let test_data = Some(AudioAnalysis {
            volume_level: Volume { peak: 0.5, rms: 0.3 },
            pitch: Pitch::Detected(440.0, 0.9),
            fft_data: None,
            timestamp: 123.456,
        });
        
        audio_setter.set(test_data.clone());
        assert_eq!(audio_observer.get(), test_data);
        
        // Test error data flow
        let test_errors = vec![AudioError::MicrophonePermissionDenied];
        errors_setter.set(test_errors.clone());
        assert_eq!(errors_observer.get(), test_errors);
        
        // Test permission state data flow
        permission_setter.set(PermissionState::Granted);
        assert_eq!(permission_observer.get(), PermissionState::Granted);
    }

    #[wasm_bindgen_test]
    fn test_interface_data_flow_isolation() {
        let interface1 = EngineToModelInterface::new();
        let interface2 = EngineToModelInterface::new();
        
        let setter1 = interface1.audio_analysis_setter();
        let observer1 = interface1.audio_analysis_observer();
        let observer2 = interface2.audio_analysis_observer();
        
        let test_data = Some(AudioAnalysis {
            volume_level: Volume { peak: 1.0, rms: 0.8 },
            pitch: Pitch::NotDetected,
            fft_data: None,
            timestamp: 789.012,
        });
        
        setter1.set(test_data.clone());
        
        // Only interface1 should have the updated data
        assert_eq!(observer1.get(), test_data);
        assert_eq!(observer2.get(), None); // interface2 should still have default
    }
}
