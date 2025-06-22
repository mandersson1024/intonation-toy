use yew::prelude::*;
use web_sys::console;
use std::rc::Rc;
use std::cell::RefCell;
use crate::services::audio_engine::AudioEngineService;
use gloo::timers::callback::Interval;

#[derive(Properties)]
pub struct AudioInspectorProps {
    pub audio_engine: Option<Rc<RefCell<AudioEngineService>>>,
    #[prop_or(1000)]
    pub update_interval_ms: u32,
    #[prop_or(false)]
    pub show_raw_buffers: bool,
    #[prop_or(false)]
    pub show_frequency_data: bool,
    #[prop_or(true)]
    pub show_pitch_data: bool,
}

impl PartialEq for AudioInspectorProps {
    fn eq(&self, other: &Self) -> bool {
        self.update_interval_ms == other.update_interval_ms 
            && self.show_raw_buffers == other.show_raw_buffers
            && self.show_frequency_data == other.show_frequency_data
            && self.show_pitch_data == other.show_pitch_data
            && self.audio_engine.as_ref().map(|e| e.as_ptr()) == other.audio_engine.as_ref().map(|e| e.as_ptr())
    }
}

#[derive(Clone, Debug)]
pub struct AudioBufferData {
    pub samples: Vec<f32>,
    pub sample_rate: f32,
    pub timestamp: f64,
    pub buffer_size: usize,
}

#[derive(Clone, Debug)]
pub struct FrequencyData {
    pub fft_bins: Vec<f32>,
    pub dominant_frequency: Option<f32>,
    pub spectral_centroid: Option<f32>,
    pub timestamp: f64,
}

#[derive(Clone, Debug)]
pub struct PitchData {
    pub detected_frequency: Option<f32>,
    pub confidence: Option<f32>,
    pub note_name: Option<String>,
    pub cents_offset: Option<f32>,
    pub timestamp: f64,
}

#[function_component(AudioInspector)]
pub fn audio_inspector(props: &AudioInspectorProps) -> Html {
    let buffer_data = use_state(|| None::<AudioBufferData>);
    let frequency_data = use_state(|| None::<FrequencyData>);
    let pitch_data = use_state(|| None::<PitchData>);
    let buffer_history = use_state(|| Vec::<AudioBufferData>::new());
    let is_monitoring = use_state(|| false);
    let _interval_handle = use_state(|| None::<Interval>);
    
    // Start/stop monitoring
    let toggle_monitoring = {
        let is_monitoring = is_monitoring.clone();
        let audio_engine = props.audio_engine.clone();
        let buffer_data = buffer_data.clone();
        let frequency_data = frequency_data.clone();
        let pitch_data = pitch_data.clone();
        let buffer_history = buffer_history.clone();
        let interval_handle = _interval_handle.clone();
        let update_interval = props.update_interval_ms;
        
        Callback::from(move |_: web_sys::MouseEvent| {
            let new_monitoring_state = !*is_monitoring;
            is_monitoring.set(new_monitoring_state);
            
            if new_monitoring_state {
                // Start monitoring - create mock data for demonstration
                let engine_clone = audio_engine.clone();
                let buffer_data_clone = buffer_data.clone();
                let frequency_data_clone = frequency_data.clone();
                let pitch_data_clone = pitch_data.clone();
                let buffer_history_clone = buffer_history.clone();
                
                let interval = Interval::new(update_interval, move || {
                    let current_time = js_sys::Date::now();
                    
                    // Generate mock audio buffer data
                    let mock_buffer = AudioBufferData {
                        samples: generate_mock_audio_samples(256),
                        sample_rate: 44100.0,
                        timestamp: current_time,
                        buffer_size: 256,
                    };
                    
                    // Generate mock frequency data
                    let mock_frequency = FrequencyData {
                        fft_bins: generate_mock_fft_data(128),
                        dominant_frequency: Some(440.0 + (current_time / 1000.0).sin() as f32 * 50.0),
                        spectral_centroid: Some(2000.0 + (current_time / 2000.0).cos() as f32 * 500.0),
                        timestamp: current_time,
                    };
                    
                    // Generate mock pitch data
                    let detected_freq = 440.0 + (current_time / 1000.0).sin() as f32 * 50.0;
                    let mock_pitch = PitchData {
                        detected_frequency: Some(detected_freq),
                        confidence: Some(0.85 + (current_time / 3000.0).sin() as f32 * 0.1),
                        note_name: Some(frequency_to_note_name(detected_freq)),
                        cents_offset: Some((current_time / 1500.0).sin() as f32 * 20.0),
                        timestamp: current_time,
                    };
                    
                    // Update state
                    buffer_data_clone.set(Some(mock_buffer.clone()));
                    frequency_data_clone.set(Some(mock_frequency));
                    pitch_data_clone.set(Some(mock_pitch));
                    
                    // Add to history (keep last 10 buffers)
                    let mut history = (*buffer_history_clone).clone();
                    history.push(mock_buffer);
                    if history.len() > 10 {
                        history.remove(0);
                    }
                    buffer_history_clone.set(history);
                });
                
                interval_handle.set(Some(interval));
                console::log_1(&"Started audio inspection monitoring".into());
            } else {
                // Stop monitoring
                interval_handle.set(None);
                console::log_1(&"Stopped audio inspection monitoring".into());
            }
        })
    };
    
    // Clear buffer history
    let clear_history = {
        let buffer_history = buffer_history.clone();
        Callback::from(move |_: web_sys::MouseEvent| {
            buffer_history.set(Vec::new());
        })
    };
    
    // Format buffer data for display
    let format_buffer_samples = |samples: &[f32], max_display: usize| {
        let display_count = samples.len().min(max_display);
        samples[..display_count]
            .iter()
            .enumerate()
            .map(|(i, sample)| format!("[{}]: {:.6}", i, sample))
            .collect::<Vec<_>>()
            .join(", ")
    };
    
    // Format frequency bins for display
    let format_frequency_bins = |bins: &[f32], sample_rate: f32, max_display: usize| {
        let display_count = bins.len().min(max_display);
        bins[..display_count]
            .iter()
            .enumerate()
            .map(|(i, magnitude)| {
                let freq = (i as f32 * sample_rate) / (bins.len() as f32 * 2.0);
                format!("{:.1}Hz: {:.3}", freq, magnitude)
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    
    html! {
        <div class="audio-inspector">
            <div class="inspector-header">
                <h3>{ "Audio Processing Inspector" }</h3>
                <div class="inspector-controls">
                    <button 
                        class={classes!("monitor-toggle", if *is_monitoring { "active" } else { "" })}
                        onclick={toggle_monitoring}
                        title="Start/stop audio inspection monitoring"
                    >
                        { if *is_monitoring { "⏹ Stop" } else { "▶ Start" } }
                    </button>
                    <button 
                        class="clear-history-btn"
                        onclick={clear_history}
                        title="Clear buffer history"
                    >
                        { "🗑 Clear History" }
                    </button>
                </div>
            </div>
            
            <div class="inspector-content">
                { if props.show_pitch_data {
                    html! {
                        <div class="inspector-section">
                            <h4>{ "🎯 Pitch Detection Results" }</h4>
                            { if let Some(pitch) = pitch_data.as_ref() {
                                html! {
                                    <div class="pitch-data">
                                        <div class="pitch-grid">
                                            <div class="pitch-item">
                                                <span class="pitch-label">{ "Frequency:" }</span>
                                                <span class="pitch-value">
                                                    { if let Some(freq) = pitch.detected_frequency {
                                                        format!("{:.2} Hz", freq)
                                                    } else {
                                                        "No pitch detected".to_string()
                                                    }}
                                                </span>
                                            </div>
                                            <div class="pitch-item">
                                                <span class="pitch-label">{ "Note:" }</span>
                                                <span class="pitch-value">
                                                    { pitch.note_name.as_ref().unwrap_or(&"N/A".to_string()) }
                                                </span>
                                            </div>
                                            <div class="pitch-item">
                                                <span class="pitch-label">{ "Confidence:" }</span>
                                                <span class="pitch-value">
                                                    { if let Some(conf) = pitch.confidence {
                                                        format!("{:.1}%", conf * 100.0)
                                                    } else {
                                                        "N/A".to_string()
                                                    }}
                                                </span>
                                            </div>
                                            <div class="pitch-item">
                                                <span class="pitch-label">{ "Cents Offset:" }</span>
                                                <span class="pitch-value">
                                                    { if let Some(cents) = pitch.cents_offset {
                                                        format!("{:+.1} cents", cents)
                                                    } else {
                                                        "N/A".to_string()
                                                    }}
                                                </span>
                                            </div>
                                        </div>
                                    </div>
                                }
                            } else {
                                html! {
                                    <div class="no-data">
                                        <p>{ "No pitch data available" }</p>
                                        <p class="hint">{ "Start monitoring to see pitch detection results" }</p>
                                    </div>
                                }
                            }}
                        </div>
                    }
                } else {
                    html! {}
                }}
                
                { if props.show_frequency_data {
                    html! {
                        <div class="inspector-section">
                            <h4>{ "📊 Frequency Domain Data" }</h4>
                            { if let Some(freq_data) = frequency_data.as_ref() {
                                html! {
                                    <div class="frequency-data">
                                        <div class="frequency-summary">
                                            <div class="freq-item">
                                                <span class="freq-label">{ "Dominant Frequency:" }</span>
                                                <span class="freq-value">
                                                    { if let Some(freq) = freq_data.dominant_frequency {
                                                        format!("{:.2} Hz", freq)
                                                    } else {
                                                        "N/A".to_string()
                                                    }}
                                                </span>
                                            </div>
                                            <div class="freq-item">
                                                <span class="freq-label">{ "Spectral Centroid:" }</span>
                                                <span class="freq-value">
                                                    { if let Some(centroid) = freq_data.spectral_centroid {
                                                        format!("{:.1} Hz", centroid)
                                                    } else {
                                                        "N/A".to_string()
                                                    }}
                                                </span>
                                            </div>
                                        </div>
                                        <div class="fft-bins">
                                            <h5>{ "FFT Bins (first 16):" }</h5>
                                            <div class="bins-display">
                                                <pre>{ format_frequency_bins(&freq_data.fft_bins, 44100.0, 16) }</pre>
                                            </div>
                                        </div>
                                    </div>
                                }
                            } else {
                                html! {
                                    <div class="no-data">
                                        <p>{ "No frequency data available" }</p>
                                        <p class="hint">{ "Start monitoring to see FFT analysis" }</p>
                                    </div>
                                }
                            }}
                        </div>
                    }
                } else {
                    html! {}
                }}
                
                { if props.show_raw_buffers {
                    html! {
                        <div class="inspector-section">
                            <h4>{ "🔢 Raw Audio Buffer Contents" }</h4>
                            { if let Some(buffer) = buffer_data.as_ref() {
                                html! {
                                    <div class="buffer-data">
                                        <div class="buffer-info">
                                            <div class="buffer-item">
                                                <span class="buffer-label">{ "Buffer Size:" }</span>
                                                <span class="buffer-value">{ buffer.buffer_size }</span>
                                            </div>
                                            <div class="buffer-item">
                                                <span class="buffer-label">{ "Sample Rate:" }</span>
                                                <span class="buffer-value">{ format!("{} Hz", buffer.sample_rate) }</span>
                                            </div>
                                            <div class="buffer-item">
                                                <span class="buffer-label">{ "Timestamp:" }</span>
                                                <span class="buffer-value">{ format!("{:.0}", buffer.timestamp) }</span>
                                            </div>
                                        </div>
                                        <div class="buffer-samples">
                                            <h5>{ "Sample Values (first 32):" }</h5>
                                            <div class="samples-display">
                                                <pre>{ format_buffer_samples(&buffer.samples, 32) }</pre>
                                            </div>
                                        </div>
                                    </div>
                                }
                            } else {
                                html! {
                                    <div class="no-data">
                                        <p>{ "No buffer data available" }</p>
                                        <p class="hint">{ "Start monitoring to see raw audio samples" }</p>
                                    </div>
                                }
                            }}
                        </div>
                    }
                } else {
                    html! {}
                }}
                
                <div class="inspector-section">
                    <h4>{ "📚 Buffer History" }</h4>
                    { if !buffer_history.is_empty() {
                        html! {
                            <div class="buffer-history">
                                <div class="history-summary">
                                    <span>{ format!("Tracking {} buffers", buffer_history.len()) }</span>
                                </div>
                                <div class="history-list">
                                    { for buffer_history.iter().enumerate().map(|(index, buffer)| {
                                        html! {
                                            <div class="history-item">
                                                <div class="history-index">{ format!("#{}", index + 1) }</div>
                                                <div class="history-info">
                                                    <span class="history-time">{ format!("{:.0}ms", buffer.timestamp) }</span>
                                                    <span class="history-size">{ format!("{} samples", buffer.samples.len()) }</span>
                                                    <span class="history-peak">
                                                        { format!("Peak: {:.3}", 
                                                            buffer.samples.iter().map(|s| s.abs()).fold(0.0, f32::max)) }
                                                    </span>
                                                </div>
                                            </div>
                                        }
                                    })}
                                </div>
                            </div>
                        }
                    } else {
                        html! {
                            <div class="no-data">
                                <p>{ "No buffer history available" }</p>
                                <p class="hint">{ "Start monitoring to track buffer history" }</p>
                            </div>
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

// Helper functions for mock data generation
fn generate_mock_audio_samples(size: usize) -> Vec<f32> {
    let mut samples = Vec::with_capacity(size);
    let time = js_sys::Date::now() / 1000.0;
    
    for i in 0..size {
        let t = (i as f64 + time * 44100.0) / 44100.0;
        // Generate a mix of frequencies for realistic audio data
        let sample = (t * 440.0 * 2.0 * std::f64::consts::PI).sin() * 0.3
                   + (t * 880.0 * 2.0 * std::f64::consts::PI).sin() * 0.2
                   + (t * 1320.0 * 2.0 * std::f64::consts::PI).sin() * 0.1;
        samples.push(sample as f32);
    }
    
    samples
}

fn generate_mock_fft_data(size: usize) -> Vec<f32> {
    let mut bins = Vec::with_capacity(size);
    let time = js_sys::Date::now() / 1000.0;
    
    for i in 0..size {
        // Generate realistic FFT magnitude data
        let freq = (i as f64 * 44100.0) / (size as f64 * 2.0);
        let magnitude = if freq < 100.0 {
            0.1 + (time + i as f64 * 0.1).sin().abs() * 0.2
        } else if freq < 1000.0 {
            0.3 + (time * 0.5 + i as f64 * 0.05).sin().abs() * 0.4
        } else if freq < 5000.0 {
            0.1 + (time * 0.3 + i as f64 * 0.02).sin().abs() * 0.2
        } else {
            0.05 + (time * 0.1 + i as f64 * 0.01).sin().abs() * 0.1
        };
        bins.push(magnitude as f32);
    }
    
    bins
}

fn frequency_to_note_name(frequency: f32) -> String {
    if frequency <= 0.0 {
        return "N/A".to_string();
    }
    
    let a4_freq = 440.0;
    let semitones_from_a4 = 12.0 * (frequency / a4_freq).log2();
    let semitone_index = (semitones_from_a4.round() as i32 + 12 * 10) % 12; // Offset to make positive
    
    let note_names = ["A", "A#", "B", "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#"];
    let octave = 4 + ((semitones_from_a4.round() as i32) / 12);
    
    format!("{}{}", note_names[semitone_index as usize], octave)
} 