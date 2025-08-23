//! Model layer - processes audio data and validates user actions

use crate::common::shared_types::{EngineUpdateResult, ModelUpdateResult, Volume, Pitch, IntonationData, TuningSystem, Scale, MidiNote, is_valid_midi_note};
use crate::presentation::PresentationLayerActions;
use crate::common::smoothing::EmaSmoother;
use crate::common::warn_log;

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    TuningSystemAlreadyActive(TuningSystem),
    TuningForkNoteAlreadySet(MidiNote),
    InvalidFrequency(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessedActions {
    pub actions: ModelLayerActions,
    pub validation_errors: Vec<ValidationError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfigureTuningForkAction {
    pub frequency: f32,
    pub volume: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ModelLayerActions {
    pub tuning_system_changes: Vec<TuningSystem>,
    pub tuning_fork_note_changes: Vec<MidiNote>,
    pub tuning_fork_configurations: Vec<ConfigureTuningForkAction>,
}

pub struct DataModel {
    tuning_system: TuningSystem,
    tuning_fork_note: MidiNote,
    current_scale: Scale,
    frequency_smoother: EmaSmoother,
    clarity_smoother: EmaSmoother,
    last_detected_pitch: Option<(f32, f32)>,
}

impl DataModel {
    pub fn create() -> Result<Self, String> {
        Ok(Self {
            tuning_system: TuningSystem::EqualTemperament,
            tuning_fork_note: crate::app_config::DEFAULT_TUNING_FORK_NOTE,
            current_scale: crate::app_config::DEFAULT_SCALE,
            frequency_smoother: EmaSmoother::new(crate::app_config::PITCH_SMOOTHING_FACTOR),
            clarity_smoother: EmaSmoother::new(crate::app_config::PITCH_SMOOTHING_FACTOR),
            last_detected_pitch: None,
        })
    }

    pub fn update(&mut self, _timestamp: f64, engine_data: EngineUpdateResult) -> ModelUpdateResult {
        let (volume, pitch) = if let Some(audio_analysis) = engine_data.audio_analysis {
            let volume = Volume {
                peak_amplitude: audio_analysis.volume_level.peak_amplitude,
                rms_amplitude: audio_analysis.volume_level.rms_amplitude,
            };
            let pitch = match audio_analysis.pitch {
                crate::common::shared_types::Pitch::Detected(frequency, clarity) => {
                    let smoothed_frequency = self.frequency_smoother.apply(frequency);
                    let smoothed_clarity = self.clarity_smoother.apply(clarity);
                    self.last_detected_pitch = Some((frequency, clarity));
                    Pitch::Detected(smoothed_frequency, smoothed_clarity)
                }
                crate::common::shared_types::Pitch::NotDetected => {
                    if let Some((last_freq, _)) = self.last_detected_pitch {
                        let smoothed_clarity = self.clarity_smoother.apply(0.0);
                        if smoothed_clarity < crate::app_config::CLARITY_THRESHOLD * 0.5 {
                            self.reset_smoothers();
                            Pitch::NotDetected
                        } else {
                            Pitch::Detected(last_freq, smoothed_clarity)
                        }
                    } else {
                        self.reset_smoothers();
                        Pitch::NotDetected
                    }
                }
            };
            
            (volume, pitch)
        } else {
            (Volume { peak_amplitude: -60.0, rms_amplitude: -60.0 }, Pitch::NotDetected)
        };
        
        let volume_peak = volume.peak_amplitude >= crate::app_config::VOLUME_PEAK_THRESHOLD;
        
        let effective_pitch = match pitch {
            Pitch::Detected(frequency, clarity) if self.frequency_to_note_and_accuracy(frequency).is_some() => {
                Pitch::Detected(frequency, clarity)
            }
            _ => Pitch::NotDetected,
        };

        let (accuracy, interval_semitones) = match effective_pitch {
            Pitch::Detected(frequency, _clarity) => {
                let (closest_midi_note, cents_offset) = self.frequency_to_note_and_accuracy(frequency).unwrap();
                let accuracy = IntonationData { closest_midi_note: Some(closest_midi_note), cents_offset };
                let interval = (closest_midi_note as i32) - (self.tuning_fork_note as i32);
                (accuracy, interval)
            }
            Pitch::NotDetected => {
                let accuracy = IntonationData { 
                    closest_midi_note: None,
                    cents_offset: 0.0,
                };
                (accuracy, 0)
            }
        };

        ModelUpdateResult {
            volume,
            volume_peak,
            pitch: effective_pitch,
            accuracy: accuracy.clone(),
            tuning_system: self.tuning_system,
            scale: self.current_scale,
            errors: engine_data.audio_errors,
            permission_state: engine_data.permission_state,
            closest_midi_note: accuracy.closest_midi_note,
            cents_offset: accuracy.cents_offset,
            interval_semitones,
            tuning_fork_note: self.tuning_fork_note,
        }
    }
    
    pub fn process_user_actions(&mut self, presentation_actions: PresentationLayerActions) -> ProcessedActions {
        let mut model_actions = ModelLayerActions::default();
        let mut validation_errors = Vec::new();
        
        for tuning_change in presentation_actions.tuning_system_changes {
            if tuning_change.tuning_system == self.tuning_system {
                validation_errors.push(ValidationError::TuningSystemAlreadyActive(tuning_change.tuning_system));
            } else {
                crate::common::dev_log!(
                    "Model layer: Tuning system changed from {:?} to {:?}",
                    self.tuning_system, tuning_change.tuning_system
                );
                self.tuning_system = tuning_change.tuning_system;
                model_actions.tuning_system_changes.push(tuning_change.tuning_system);
            }
        }
        
        for tuning_fork_adjustment in presentation_actions.tuning_fork_adjustments {
            let midi_note = tuning_fork_adjustment.note;
            if midi_note == self.tuning_fork_note {
                validation_errors.push(ValidationError::TuningForkNoteAlreadySet(midi_note));
            } else {
                crate::common::dev_log!(
                    "Model layer: Tuning fork changed from {} to {}",
                    self.tuning_fork_note, midi_note
                );
                self.tuning_fork_note = midi_note;
                model_actions.tuning_fork_note_changes.push(midi_note);
            }
        }
        
        for scale_change in presentation_actions.scale_changes {
            if scale_change.scale != self.current_scale {
                crate::common::dev_log!(
                    "Model layer: Scale changed from {:?} to {:?}",
                    self.current_scale, scale_change.scale
                );
                self.current_scale = scale_change.scale;
            }
        }
        
        crate::common::dev_log!("MODEL: Processing {} tuning fork audio configurations", presentation_actions.tuning_fork_configurations.len());
        for tuning_fork_config in presentation_actions.tuning_fork_configurations {
            crate::common::dev_log!("MODEL: Processing tuning fork audio config");
            
            if tuning_fork_config.frequency <= 0.0 {
                let error = ValidationError::InvalidFrequency(tuning_fork_config.frequency);
                crate::common::warn_log!("Tuning fork audio configuration validation failed: {:?}", error);
                validation_errors.push(error);
            } else {
                model_actions.tuning_fork_configurations.push(
                    ConfigureTuningForkAction {
                        frequency: tuning_fork_config.frequency,
                        volume: tuning_fork_config.volume,
                    }
                );
                crate::common::dev_log!("MODEL: ✓ Tuning fork audio configuration validated and queued for engine execution");
            }
        }
        
        ProcessedActions { actions: model_actions, validation_errors }
    }

    fn reset_smoothers(&mut self) {
        self.last_detected_pitch = None;
        self.frequency_smoother.reset();
        self.clarity_smoother.reset();
    }
    
    fn frequency_to_note_and_accuracy(&self, frequency: f32) -> Option<(MidiNote, f32)> {
        if frequency <= 0.0 {
            warn_log!("[MODEL] Invalid frequency for note conversion: {}", frequency);
            return None;
        }
        
        let root_pitch = crate::common::music_theory::midi_note_to_standard_frequency(self.tuning_fork_note);
        let interval_result = crate::common::music_theory::frequency_to_interval_semitones_scale_aware(
            self.tuning_system,
            root_pitch,
            frequency,
            self.current_scale,
        );
        
        let raw_midi_note = self.tuning_fork_note as i32 + interval_result.semitones;
        
        if !(0..=127).contains(&raw_midi_note) {
            return None;
        }
        
        let midi_note = raw_midi_note as u8;
        if !is_valid_midi_note(midi_note) {
            return None;
        }
        
        Some((midi_note, interval_result.cents))
    }
    
}

