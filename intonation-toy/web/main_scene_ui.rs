#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use web_sys::{window, HtmlSelectElement, HtmlInputElement, EventTarget};

#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::sync::atomic::{AtomicU8, Ordering};

#[cfg(target_arch = "wasm32")]
use crate::common::dev_log;
#[cfg(target_arch = "wasm32")]
use crate::shared_types::{TuningSystem, MidiNote, Scale, increment_midi_note, decrement_midi_note};

// Global state for current root note - initialized to default from config
#[cfg(target_arch = "wasm32")]
static CURRENT_ROOT_NOTE: AtomicU8 = AtomicU8::new(crate::app_config::DEFAULT_ROOT_NOTE);


// Global state for tuning fork volume slider position (0-100)
#[cfg(target_arch = "wasm32")]
static CURRENT_TUNING_FORK_VOLUME_POSITION: AtomicU8 = AtomicU8::new(0);

// Global state for zoom level slider position (50-150, representing percentage)
#[cfg(target_arch = "wasm32")]
static CURRENT_ZOOM_LEVEL: AtomicU8 = AtomicU8::new(95);

/// Convert slider position (0-100) to amplitude (0.0-1.0) using dual-scale approach
/// - 0-20%: Linear scaling from 0.0 to 0.01 amplitude
/// - 20-100%: dB scaling from -40dB to 0dB
#[cfg(target_arch = "wasm32")]
fn slider_position_to_amplitude(position: f32) -> f32 {
    if position <= 0.0 {
        0.0
    } else if position <= 20.0 {
        // Linear scaling: 0-20% maps to 0.0-0.01 amplitude
        position * 0.01 / 20.0
    } else {
        // dB scaling: 20-100% maps to -40dB to 0dB
        let db = -40.0 + (position - 20.0) * 40.0 / 80.0;
        10.0_f32.powf(db / 20.0)
    }
}

/// Convert amplitude (0.0-1.0) to slider position (0-100) for UI synchronization
#[cfg(target_arch = "wasm32")]
fn amplitude_to_slider_position(amplitude: f32) -> f32 {
    if amplitude <= 0.0 {
        0.0
    } else if amplitude <= 0.01 {
        // Linear range: 0.0-0.01 amplitude maps to 0-20% position
        amplitude * 20.0 / 0.01
    } else {
        // dB range: calculate dB from amplitude and map to 20-100% position
        let db = 20.0 * amplitude.log10();
        let db = db.max(-40.0).min(0.0); // Clamp to valid range
        20.0 + (db + 40.0) * 80.0 / 40.0
    }
}

/// Convert slider position to dB display string
/// - 0%: Shows "-∞ dB"
/// - 0-20%: Calculates dB from amplitude
/// - 20-100%: Maps directly to dB scale
#[cfg(target_arch = "wasm32")]
fn slider_position_to_db_display(position: f32) -> String {
    if position <= 0.0 {
        "-∞ dB".to_string()
    } else if position <= 20.0 {
        // Calculate dB from the amplitude in the linear range
        let amplitude = slider_position_to_amplitude(position);
        if amplitude > 0.0 {
            let db = 20.0 * amplitude.log10();
            format!("{:.1} dB", db)
        } else {
            "-∞ dB".to_string()
        }
    } else {
        // Direct dB mapping for 20-100% range
        let db = -40.0 + (position - 20.0) * 40.0 / 80.0;
        format!("{:.1} dB", db)
    }
}

/// Format a MIDI note number as a string (e.g., 60 -> "C4")
#[cfg(target_arch = "wasm32")]
fn format_midi_note(midi_note: MidiNote) -> String {
    crate::shared_types::midi_note_to_name(midi_note)
}

/// Convert zoom slider percentage (50-150) to zoom factor (0.95-1.5)
/// Bottom position (50) maps to PITCH_VISUALIZATION_ZOOM_DEFAULT (0.95)
/// Top position (150) maps to PITCH_VISUALIZATION_ZOOM_MAX (1.5)
#[cfg(target_arch = "wasm32")]
fn slider_percentage_to_zoom_factor(percentage: f32) -> f32 {
    let clamped = percentage.max(50.0).min(150.0);
    // Map 50-150 to 0.95-1.5
    let normalized = (clamped - 50.0) / 100.0; // 0.0 to 1.0
    let zoom_range = crate::app_config::PITCH_VISUALIZATION_ZOOM_MAX - crate::app_config::PITCH_VISUALIZATION_ZOOM_DEFAULT;
    crate::app_config::PITCH_VISUALIZATION_ZOOM_DEFAULT + normalized * zoom_range
}

/// Convert zoom factor (0.95-1.5) to slider percentage (50-150)
#[cfg(target_arch = "wasm32")]
fn zoom_factor_to_slider_percentage(factor: f32) -> f32 {
    let zoom_range = crate::app_config::PITCH_VISUALIZATION_ZOOM_MAX - crate::app_config::PITCH_VISUALIZATION_ZOOM_DEFAULT;
    let normalized = (factor - crate::app_config::PITCH_VISUALIZATION_ZOOM_DEFAULT) / zoom_range;
    let percentage = 50.0 + normalized * 100.0;
    percentage.max(50.0).min(150.0)
}

/// Format zoom percentage for display (e.g., "95%")
#[cfg(target_arch = "wasm32")]
fn format_zoom_percentage(percentage: f32) -> String {
    format!("{:.0}%", percentage)
}


#[cfg(target_arch = "wasm32")]
pub fn setup_main_scene_ui() {
    let Some(window) = window() else {
        dev_log!("Failed to get window");
        return;
    };

    let Some(document) = window.document() else {
        dev_log!("Failed to get document");
        return;
    };

    // Verify that essential HTML elements exist and set initial values
    if let Some(root_note_display) = document.get_element_by_id("root-note-display") {
        let default_note_name = format_midi_note(crate::app_config::DEFAULT_ROOT_NOTE);
        root_note_display.set_text_content(Some(&default_note_name));
    } else {
        dev_log!("Warning: root-note-display element not found in HTML");
    }

    if let Some(volume_display) = document.get_element_by_id("tuning-fork-volume-display") {
        volume_display.set_text_content(Some(&slider_position_to_db_display(0.0)));
    } else {
        dev_log!("Warning: tuning-fork-volume-display element not found in HTML");
    }

    if let Some(volume_slider) = document.get_element_by_id("tuning-fork-volume") {
        if let Some(html_slider) = volume_slider.dyn_ref::<HtmlInputElement>() {
            html_slider.set_value("0");
        }
    } else {
        dev_log!("Warning: tuning-fork-volume element not found in HTML");
    }

    // Initialize zoom control elements
    let default_zoom_percentage = zoom_factor_to_slider_percentage(crate::app_config::PITCH_VISUALIZATION_ZOOM_DEFAULT);
    CURRENT_ZOOM_LEVEL.store(default_zoom_percentage as u8, Ordering::Relaxed);
    
    if let Some(zoom_slider) = document.get_element_by_id("zoom-slider") {
        if let Some(html_slider) = zoom_slider.dyn_ref::<HtmlInputElement>() {
            html_slider.set_value(&default_zoom_percentage.to_string());
        }
    } else {
        dev_log!("Warning: zoom-slider element not found in HTML");
    }
    
    if let Some(zoom_display) = document.get_element_by_id("zoom-display") {
        zoom_display.set_text_content(Some(&format_zoom_percentage(default_zoom_percentage)));
    } else {
        dev_log!("Warning: zoom-display element not found in HTML");
    }


    // Verify other essential elements exist
    if document.get_element_by_id("root-note-plus").is_none() {
        dev_log!("Warning: root-note-plus element not found in HTML");
    }
    if document.get_element_by_id("root-note-minus").is_none() {
        dev_log!("Warning: root-note-minus element not found in HTML");
    }
    if document.get_element_by_id("tuning-system-select").is_none() {
        dev_log!("Warning: tuning-system-select element not found in HTML");
    }
    if document.get_element_by_id("scale-select").is_none() {
        dev_log!("Warning: scale-select element not found in HTML");
    }
}

#[cfg(target_arch = "wasm32")]
pub fn cleanup_main_scene_ui() {
    // No cleanup needed since we're now using static HTML elements
    // The HTML elements remain in the DOM and can be reused
}

#[cfg(target_arch = "wasm32")]
pub fn setup_event_listeners(presenter: Rc<RefCell<crate::presentation::Presenter>>) {
    let Some(window) = window() else {
        dev_log!("Failed to get window for event listeners");
        return;
    };

    let Some(document) = window.document() else {
        dev_log!("Failed to get document for event listeners");
        return;
    };

    // Set up plus button event listener
    if let Some(plus_button) = document.get_element_by_id("root-note-plus") {
        let presenter_clone = presenter.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            let current_root_note = CURRENT_ROOT_NOTE.load(Ordering::Relaxed);
            if let Some(new_root) = increment_midi_note(current_root_note) {
                if let Ok(mut presenter_mut) = presenter_clone.try_borrow_mut() {
                    presenter_mut.on_root_note_adjusted(new_root);
                    
                    // Also update root note audio frequency
                    let position = CURRENT_TUNING_FORK_VOLUME_POSITION.load(Ordering::Relaxed) as f32;
                    let amplitude = slider_position_to_amplitude(position);
                    presenter_mut.on_root_note_audio_configured(true, new_root, amplitude);
                }
            }
        }) as Box<dyn FnMut(_)>);

        if let Some(event_target) = plus_button.dyn_ref::<EventTarget>() {
            if let Err(err) = event_target.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()) {
                dev_log!("Failed to add click listener to plus button: {:?}", err);
            }
        }
        closure.forget();
    } else {
        dev_log!("Failed to find root-note-plus button");
    }

    // Set up minus button event listener
    if let Some(minus_button) = document.get_element_by_id("root-note-minus") {
        let presenter_clone = presenter.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            let current_root_note = CURRENT_ROOT_NOTE.load(Ordering::Relaxed);
            if let Some(new_root) = decrement_midi_note(current_root_note) {
                if let Ok(mut presenter_mut) = presenter_clone.try_borrow_mut() {
                    presenter_mut.on_root_note_adjusted(new_root);
                    
                    // Also update root note audio frequency
                    let position = CURRENT_TUNING_FORK_VOLUME_POSITION.load(Ordering::Relaxed) as f32;
                    let amplitude = slider_position_to_amplitude(position);
                    presenter_mut.on_root_note_audio_configured(true, new_root, amplitude);
                }
            }
        }) as Box<dyn FnMut(_)>);

        if let Some(event_target) = minus_button.dyn_ref::<EventTarget>() {
            if let Err(err) = event_target.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()) {
                dev_log!("Failed to add click listener to minus button: {:?}", err);
            }
        }
        closure.forget();
    } else {
        dev_log!("Failed to find root-note-minus button");
    }

    // Set up tuning system dropdown event listener
    if let Some(tuning_select) = document.get_element_by_id("tuning-system-select") {
        let presenter_clone = presenter.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            if let Some(current_window) = web_sys::window() {
                if let Some(document) = current_window.document() {
                    if let Some(select_element) = document.get_element_by_id("tuning-system-select") {
                if let Some(html_select) = select_element.dyn_ref::<HtmlSelectElement>() {
                    let value = html_select.value();
                    let tuning_system = match value.as_str() {
                        "equal" => TuningSystem::EqualTemperament,
                        "just" => TuningSystem::JustIntonation,
                        _ => {
                            dev_log!("Unknown tuning system value: {}", value);
                            return;
                        }
                    };
                        presenter_clone.borrow_mut().on_tuning_system_changed(tuning_system);
                    }
                }
                }
            }
        }) as Box<dyn FnMut(_)>);

        if let Some(event_target) = tuning_select.dyn_ref::<EventTarget>() {
            if let Err(err) = event_target.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref()) {
                dev_log!("Failed to add change listener to tuning dropdown: {:?}", err);
            }
        }
        closure.forget();
    } else {
        dev_log!("Failed to find tuning-system-select dropdown");
    }

    // Set up scale dropdown event listener
    if let Some(scale_select) = document.get_element_by_id("scale-select") {
        let presenter_clone = presenter.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            if let Some(current_window) = web_sys::window() {
                if let Some(document) = current_window.document() {
                    if let Some(select_element) = document.get_element_by_id("scale-select") {
                if let Some(html_select) = select_element.dyn_ref::<HtmlSelectElement>() {
                    let value = html_select.value();
                    let scale = match value.as_str() {
                        "chromatic" => Scale::Chromatic,
                        "major" => Scale::Major,
                        "minor" => Scale::Minor,
                        "major_pentatonic" => Scale::MajorPentatonic,
                        "minor_pentatonic" => Scale::MinorPentatonic,
                        _ => {
                            dev_log!("Unknown scale value: {}", value);
                            return;
                        }
                    };
                        presenter_clone.borrow_mut().on_scale_changed(scale);
                    }
                }
                }
            }
        }) as Box<dyn FnMut(_)>);

        if let Some(event_target) = scale_select.dyn_ref::<EventTarget>() {
            if let Err(err) = event_target.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref()) {
                dev_log!("Failed to add change listener to scale dropdown: {:?}", err);
            }
        }
        closure.forget();
    } else {
        dev_log!("Failed to find scale-select dropdown");
    }

    // Set up tuning fork volume slider event listener
    if let Some(slider) = document.get_element_by_id("tuning-fork-volume") {
        let presenter_clone = presenter.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            if let Some(current_window) = web_sys::window() {
                if let Some(document) = current_window.document() {
                    if let Some(slider_element) = document.get_element_by_id("tuning-fork-volume") {
                        if let Some(html_slider) = slider_element.dyn_ref::<HtmlInputElement>() {
                            if let Ok(position) = html_slider.value().parse::<f32>() {
                                CURRENT_TUNING_FORK_VOLUME_POSITION.store(position as u8, Ordering::Relaxed);
                                
                                // Convert position to amplitude
                                let amplitude = slider_position_to_amplitude(position);
                                
                                // Update volume display with dB value
                                if let Some(display_element) = document.get_element_by_id("tuning-fork-volume-display") {
                                    display_element.set_text_content(Some(&slider_position_to_db_display(position)));
                                }
                                
                                // Update audio
                                let current_root_note = CURRENT_ROOT_NOTE.load(Ordering::Relaxed);
                                presenter_clone.borrow_mut().on_root_note_audio_configured(true, current_root_note, amplitude);
                            }
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        if let Some(event_target) = slider.dyn_ref::<EventTarget>() {
            if let Err(err) = event_target.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref()) {
                dev_log!("Failed to add input listener to volume slider: {:?}", err);
            }
        }
        closure.forget();
    } else {
        dev_log!("Failed to find tuning-fork-volume slider");
    }

    // Set up zoom slider event listener
    if let Some(slider) = document.get_element_by_id("zoom-slider") {
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            if let Some(current_window) = web_sys::window() {
                if let Some(document) = current_window.document() {
                    if let Some(slider_element) = document.get_element_by_id("zoom-slider") {
                        if let Some(html_slider) = slider_element.dyn_ref::<HtmlInputElement>() {
                            if let Ok(percentage) = html_slider.value().parse::<f32>() {
                                // Store the percentage value
                                CURRENT_ZOOM_LEVEL.store(percentage as u8, Ordering::Relaxed);
                                
                                // Update zoom display with percentage
                                if let Some(display_element) = document.get_element_by_id("zoom-display") {
                                    display_element.set_text_content(Some(&format_zoom_percentage(percentage)));
                                }
                                
                                // Note: The actual zoom factor will be retrieved when needed
                                // using slider_percentage_to_zoom_factor(percentage)
                            }
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        if let Some(event_target) = slider.dyn_ref::<EventTarget>() {
            if let Err(err) = event_target.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref()) {
                dev_log!("Failed to add input listener to zoom slider: {:?}", err);
            }
        }
        closure.forget();
    } else {
        dev_log!("Failed to find zoom-slider");
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn setup_main_scene_ui() {
    // No-op for non-WASM targets
}

#[cfg(not(target_arch = "wasm32"))]
pub fn cleanup_main_scene_ui() {
    // No-op for non-WASM targets
}

/// Synchronize UI elements with current presenter state
/// 
/// This function updates the UI to reflect the current root note and tuning system
/// values from the presenter, ensuring the UI stays in sync when values change
/// from sources other than direct user interaction.
/// 
/// # Arguments
/// 
/// * `root_note` - The current root note from the presenter
/// * `tuning_system` - The current tuning system from the presenter
/// * `scale` - The current scale from the presenter
/// * `root_note_audio_enabled` - The current root note audio state
#[cfg(target_arch = "wasm32")]
pub fn sync_ui_with_presenter_state(model_data: &crate::shared_types::ModelUpdateResult) {
    let Some(window) = window() else {
        return;
    };

    let Some(document) = window.document() else {
        return;
    };

    // Update stored root note state
    CURRENT_ROOT_NOTE.store(model_data.root_note, Ordering::Relaxed);

    // Update root note display
    if let Some(display) = document.get_element_by_id("root-note-display") {
        let formatted_note = format_midi_note(model_data.root_note);
        display.set_text_content(Some(&formatted_note));
    }

    // Update tuning system dropdown selection
    if let Some(select_element) = document.get_element_by_id("tuning-system-select") {
        if let Some(html_select) = select_element.dyn_ref::<HtmlSelectElement>() {
            let value = match model_data.tuning_system {
                TuningSystem::EqualTemperament => "equal",
                TuningSystem::JustIntonation => "just",
            };
            html_select.set_value(value);
        }
    }

    // Update scale dropdown selection
    if let Some(select_element) = document.get_element_by_id("scale-select") {
        if let Some(html_select) = select_element.dyn_ref::<HtmlSelectElement>() {
            let value = match model_data.scale {
                Scale::Chromatic => "chromatic",
                Scale::Major => "major",
                Scale::Minor => "minor",
                Scale::MajorPentatonic => "major_pentatonic",
                Scale::MinorPentatonic => "minor_pentatonic",
            };
            html_select.set_value(value);
        }
    }

    // Update volume slider and display
    let current_position = CURRENT_TUNING_FORK_VOLUME_POSITION.load(Ordering::Relaxed) as f32;
    if let Some(slider_element) = document.get_element_by_id("tuning-fork-volume") {
        if let Some(html_slider) = slider_element.dyn_ref::<HtmlInputElement>() {
            html_slider.set_value(&current_position.to_string());
        }
    }
    if let Some(display_element) = document.get_element_by_id("tuning-fork-volume-display") {
        display_element.set_text_content(Some(&slider_position_to_db_display(current_position)));
    }

    // Update zoom slider and display
    let current_zoom_percentage = CURRENT_ZOOM_LEVEL.load(Ordering::Relaxed) as f32;
    if let Some(slider_element) = document.get_element_by_id("zoom-slider") {
        if let Some(html_slider) = slider_element.dyn_ref::<HtmlInputElement>() {
            html_slider.set_value(&current_zoom_percentage.to_string());
        }
    }
    if let Some(display_element) = document.get_element_by_id("zoom-display") {
        display_element.set_text_content(Some(&format_zoom_percentage(current_zoom_percentage)));
    }
}

/// Get the current zoom factor for pitch visualization
/// Returns a value between 0.5 and 1.5 representing the zoom level
#[cfg(target_arch = "wasm32")]
pub fn get_current_zoom_factor() -> f32 {
    let percentage = CURRENT_ZOOM_LEVEL.load(Ordering::Relaxed) as f32;
    slider_percentage_to_zoom_factor(percentage)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_current_zoom_factor() -> f32 {
    // Return default zoom factor for non-WASM targets
    crate::app_config::PITCH_VISUALIZATION_ZOOM_DEFAULT
}

#[cfg(not(target_arch = "wasm32"))]
pub fn setup_event_listeners(_presenter: std::rc::Rc<std::cell::RefCell<crate::presentation::Presenter>>) {
    // No-op for non-WASM targets
}

#[cfg(not(target_arch = "wasm32"))]
pub fn sync_ui_with_presenter_state(_model_data: &crate::shared_types::ModelUpdateResult) {
    // No-op for non-WASM targets
}