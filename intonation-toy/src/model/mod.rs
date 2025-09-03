//! Model layer - processes audio data and validates user actions

use crate::common::shared_types::{EngineUpdateResult, ModelUpdateResult, Volume, Pitch, TuningSystem, Scale, MidiNote};
use crate::presentation::PresentationLayerActions;
use crate::common::smoothing::EmaSmoother;

#[derive(Debug, Clone, PartialEq)]
pub struct ConfigureTuningForkAction {
    pub frequency: f32,
    pub volume: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ModelLayerActions {
    pub tuning_fork_configuration: Option<ConfigureTuningForkAction>,
}

impl ModelLayerActions {
    /// Check if there are any actions to process
    pub fn has_actions(&self) -> bool {
        self.tuning_fork_configuration.is_some()
    }
}

pub struct DataModel {
    tuning_system: TuningSystem,
    tuning_fork_note: MidiNote,
    current_scale: Scale,
    frequency_smoother: EmaSmoother,
    clarity_smoother: EmaSmoother,
    last_detected_pitch: Option<(f32, f32)>,
}

impl Default for DataModel {
    fn default() -> Self {
        Self {
            tuning_system: TuningSystem::EqualTemperament,
            tuning_fork_note: crate::app_config::DEFAULT_TUNING_FORK_NOTE,
            current_scale: crate::app_config::DEFAULT_SCALE,
            frequency_smoother: EmaSmoother::new(crate::app_config::PITCH_SMOOTHING_FACTOR),
            clarity_smoother: EmaSmoother::new(crate::app_config::PITCH_SMOOTHING_FACTOR),
            last_detected_pitch: None,
        }
    }
}

impl DataModel {
    pub fn new(tuning_fork_note: MidiNote, tuning_system: TuningSystem, scale: Scale) -> Self {
        Self {
            tuning_system,
            tuning_fork_note,
            current_scale: scale,
            frequency_smoother: EmaSmoother::new(crate::app_config::PITCH_SMOOTHING_FACTOR),
            clarity_smoother: EmaSmoother::new(crate::app_config::PITCH_SMOOTHING_FACTOR),
            last_detected_pitch: None,
        }
    }

    pub fn update(&mut self, engine_data: EngineUpdateResult) -> ModelUpdateResult {
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
            (Volume { peak_amplitude: 0.0, rms_amplitude: 0.0 }, Pitch::NotDetected)
        };
        
        let is_peaking = volume.peak_amplitude >= crate::app_config::VOLUME_PEAK_THRESHOLD;
        
        let midi_note_result = match pitch {
            Pitch::Detected(frequency, _) => crate::common::music_theory::frequency_to_midi_note_and_cents(
                frequency,
                self.tuning_fork_note,
                self.tuning_system,
                self.current_scale,
            ),
            _ => None,
        };

        let (closest_midi_note, cents_offset, interval_semitones) = match midi_note_result {
            Some((midi_note, cents)) => {
                let interval = (midi_note as i32) - (self.tuning_fork_note as i32);
                (Some(midi_note), cents, interval)
            }
            None => (None, 0.0, 0),
        };

        ModelUpdateResult {
            volume,
            is_peaking,
            pitch,
            tuning_system: self.tuning_system,
            scale: self.current_scale,
            closest_midi_note,
            cents_offset,
            interval_semitones,
            tuning_fork_note: self.tuning_fork_note,
        }
    }
    
    pub fn process_user_actions(&mut self, presentation_actions: PresentationLayerActions) -> ModelLayerActions {
        let mut model_actions = ModelLayerActions::default();
        
        if let Some(tuning_change) = presentation_actions.tuning_system_change {
            if tuning_change.tuning_system != self.tuning_system {
                crate::common::dev_log!(
                    "Model layer: Tuning system changed from {:?} to {:?}",
                    self.tuning_system, tuning_change.tuning_system
                );
                self.tuning_system = tuning_change.tuning_system;
            }
        }
        
        if let Some(scale_change) = presentation_actions.scale_change {
            if scale_change.scale != self.current_scale {
                crate::common::dev_log!(
                    "Model layer: Scale changed from {:?} to {:?}",
                    self.current_scale, scale_change.scale
                );
                self.current_scale = scale_change.scale;
            }
        }
        
        if let Some(tuning_fork_config) = &presentation_actions.tuning_fork_configuration {
            if tuning_fork_config.note != self.tuning_fork_note {
                crate::common::dev_log!(
                    "Model layer: Tuning fork changed from {} to {}",
                    self.tuning_fork_note, tuning_fork_config.note
                );
                self.tuning_fork_note = tuning_fork_config.note;
            }
            
            model_actions.tuning_fork_configuration = Some(
                ConfigureTuningForkAction {
                    frequency: crate::common::music_theory::midi_note_to_standard_frequency(tuning_fork_config.note),
                    volume: tuning_fork_config.volume,
                }
            );
        }
        
        model_actions
    }

    fn reset_smoothers(&mut self) {
        self.last_detected_pitch = None;
        self.frequency_smoother.reset();
        self.clarity_smoother.reset();
    }
    
}

