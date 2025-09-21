#![cfg(target_arch = "wasm32")]

//! Model layer - processes audio data and validates user actions

use crate::common::shared_types::{EngineUpdateResult, ModelUpdateResult, Volume, Pitch, TuningSystem, Scale, MidiNote};
use crate::presentation::PresentationLayerActions;
use crate::common::smoothing::EmaSmoother;
use crate::common::adaptive_ema::AdaptiveEMA;

#[derive(Debug, Clone, PartialEq)]
pub struct ConfigureTonalCenterAction {
    pub frequency: f32,
    pub volume: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ModelLayerActions {
    pub tonal_center_configuration: Option<ConfigureTonalCenterAction>,
}

impl ModelLayerActions {
    /// Check if there are any actions to process
    pub fn has_actions(&self) -> bool {
        self.tonal_center_configuration.is_some()
    }
}

pub struct DataModel {
    tuning_system: TuningSystem,
    tonal_center_note: MidiNote,
    current_scale: Scale,
    frequency_smoother: Box<dyn PitchSmoother>,
    last_detected_pitch: Option<f32>,
}

/// Trait for pitch smoothing algorithms
trait PitchSmoother: Send {
    fn apply(&mut self, value: f32) -> f32;
    fn reset(&mut self);
}

impl PitchSmoother for EmaSmoother {
    fn apply(&mut self, value: f32) -> f32 {
        self.apply(value)
    }

    fn reset(&mut self) {
        self.reset()
    }
}

impl PitchSmoother for AdaptiveEMA {
    fn apply(&mut self, value: f32) -> f32 {
        self.apply(value)
    }

    fn reset(&mut self) {
        self.reset()
    }
}

/// Create a smoother based on configuration
fn create_smoother() -> Box<dyn PitchSmoother> {
    if crate::app_config::USE_ADAPTIVE_EMA {
        let mut ema = AdaptiveEMA::new(
            crate::app_config::ADAPTIVE_EMA_ALPHA_MIN,
            crate::app_config::ADAPTIVE_EMA_ALPHA_MAX,
            crate::app_config::ADAPTIVE_EMA_D,
            crate::app_config::ADAPTIVE_EMA_S,
        );

        if crate::app_config::ADAPTIVE_EMA_USE_MEDIAN3 {
            ema = ema.with_median3(true);
        }

        if crate::app_config::ADAPTIVE_EMA_USE_HAMPEL {
            ema = ema.with_hampel(
                true,
                crate::app_config::ADAPTIVE_EMA_HAMPEL_WINDOW,
                crate::app_config::ADAPTIVE_EMA_HAMPEL_NSIGMA,
            );
        }

        if crate::app_config::ADAPTIVE_EMA_DEADBAND > 0.0 {
            ema = ema.with_deadband(crate::app_config::ADAPTIVE_EMA_DEADBAND);
        }

        ema = ema.with_hysteresis(
            crate::app_config::ADAPTIVE_EMA_HYSTERESIS_DOWN,
            crate::app_config::ADAPTIVE_EMA_HYSTERESIS_UP,
        );

        Box::new(ema)
    } else {
        Box::new(EmaSmoother::new(crate::app_config::PITCH_SMOOTHING_FACTOR))
    }
}

impl Default for DataModel {
    fn default() -> Self {
        Self {
            tuning_system: TuningSystem::EqualTemperament,
            tonal_center_note: crate::app_config::DEFAULT_TONAL_CENTER_NOTE,
            current_scale: crate::app_config::DEFAULT_SCALE,
            frequency_smoother: create_smoother(),
            last_detected_pitch: None,
        }
    }
}

impl DataModel {
    pub fn new(tonal_center_note: MidiNote, tuning_system: TuningSystem, scale: Scale) -> Self {
        Self {
            tuning_system,
            tonal_center_note,
            current_scale: scale,
            frequency_smoother: create_smoother(),
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
                crate::common::shared_types::Pitch::Detected(frequency) => {
                    let smoothed_frequency = self.frequency_smoother.apply(frequency);
                    self.last_detected_pitch = Some(frequency);
                    Pitch::Detected(smoothed_frequency)
                }
                crate::common::shared_types::Pitch::NotDetected => {
                    if let Some(last_freq) = self.last_detected_pitch {
                        // Continue using last frequency for a smooth decay
                        Pitch::Detected(last_freq)
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
            Pitch::Detected(frequency) => crate::common::music_theory::frequency_to_midi_note_and_cents(
                frequency,
                self.tonal_center_note,
                self.tuning_system,
                self.current_scale,
            ),
            _ => None,
        };

        let (closest_midi_note, cents_offset, interval_semitones) = match midi_note_result {
            Some((midi_note, cents)) => {
                let interval = (midi_note as i32) - (self.tonal_center_note as i32);
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
            tonal_center_note: self.tonal_center_note,
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
        
        if let Some(tonal_center_config) = &presentation_actions.tonal_center_configuration {
            if tonal_center_config.note != self.tonal_center_note {
                crate::common::dev_log!(
                    "Model layer: Tonal center changed from {} to {}",
                    self.tonal_center_note, tonal_center_config.note
                );
                self.tonal_center_note = tonal_center_config.note;
            }
            
            model_actions.tonal_center_configuration = Some(
                ConfigureTonalCenterAction {
                    frequency: crate::common::music_theory::midi_note_to_standard_frequency(tonal_center_config.note),
                    volume: tonal_center_config.volume,
                }
            );
        }
        
        model_actions
    }

    fn reset_smoothers(&mut self) {
        self.last_detected_pitch = None;
        self.frequency_smoother.reset();
    }
    
}

