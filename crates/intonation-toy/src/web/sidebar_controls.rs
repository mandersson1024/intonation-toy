#![cfg(target_arch = "wasm32")]

use {
    wasm_bindgen::JsCast,
    wasm_bindgen::closure::Closure,
    web_sys::{window, HtmlSelectElement, HtmlInputElement, EventTarget},
    std::rc::Rc,
    std::cell::RefCell,
    std::sync::atomic::{AtomicU8, Ordering},
    crate::common::dev_log,
    crate::common::shared_types::{TuningSystem, Scale, DisplayRange, increment_midi_note, decrement_midi_note},
    crate::web::storage,
};

// These statics are needed because the tonal center controls (plus/minus buttons and volume slider)
// interact with each other in a way that requires shared state:
// 1. The plus/minus buttons need to know the current note to calculate next/previous values
// 2. When the note changes, we need to send both the new note AND the current volume to the presenter
// 3. The volume position must be preserved when the note changes
// Unlike the dropdown controls (scale/tuning system) which maintain their own state in the DOM,
// these controls need coordinated state management to work together properly.
static CURRENT_TONAL_CENTER_NOTE: AtomicU8 = AtomicU8::new(crate::app_config::DEFAULT_TONAL_CENTER_NOTE);

static CURRENT_TONAL_CENTER_VOLUME_POSITION: AtomicU8 = AtomicU8::new(0);

// Default volume position when unmuting
const DEFAULT_VOLUME_POSITION: u8 = 40;

// Remembered volume position for toggle functionality
static REMEMBERED_VOLUME_POSITION: AtomicU8 = AtomicU8::new(DEFAULT_VOLUME_POSITION);

// Track last saved configuration to avoid saving every frame
static LAST_SAVED_CONFIG: std::sync::Mutex<Option<(u8, TuningSystem, Scale, DisplayRange)>> = std::sync::Mutex::new(None);

// Track current display range
static CURRENT_DISPLAY_RANGE: std::sync::Mutex<DisplayRange> = std::sync::Mutex::new(crate::app_config::DEFAULT_DISPLAY_RANGE);

fn slider_position_to_amplitude(position: f32) -> f32 {
    if position <= 0.0 {
        0.0
    } else if position <= 20.0 {
        position * 0.01 / 20.0
    } else {
        let db = -40.0 + (position - 20.0) * 40.0 / 80.0;
        10.0_f32.powf(db / 20.0)
    }
}

fn update_volume_icon_state(is_muted: bool) {
    let Some(window) = window() else { return; };
    let Some(document) = window.document() else { return; };
    let Some(icon) = document.get_element_by_id("volume-icon") else { return; };

    if is_muted {
        let _ = icon.class_list().add_1("muted");
    } else {
        let _ = icon.class_list().remove_1("muted");
    }
}

fn slider_position_to_db_display(position: f32) -> String {
    if position <= 0.0 {
        "-∞ dB".to_string()
    } else if position <= 20.0 {
        let amplitude = slider_position_to_amplitude(position);
        if amplitude > 0.0 {
            let db = 20.0 * amplitude.log10();
            format!("{:.0} dB", db)
        } else {
            "-∞ dB".to_string()
        }
    } else {
        let db = -40.0 + (position - 20.0) * 40.0 / 80.0;
        format!("{:.0} dB", db)
    }
}

pub fn set_initial_display_range(display_range: DisplayRange) {
    if let Ok(mut current) = CURRENT_DISPLAY_RANGE.try_lock() {
        *current = display_range;
    }
}

pub fn setup_sidebar_controls() {
    let Some(window) = window() else {
        dev_log!("Failed to get window");
        return;
    };

    let Some(document) = window.document() else {
        dev_log!("Failed to get document");
        return;
    };

    if let Some(tonal_center_display) = document.get_element_by_id("tonal-center-display") {
        let default_note_name = crate::common::shared_types::midi_note_to_name(crate::app_config::DEFAULT_TONAL_CENTER_NOTE);
        tonal_center_display.set_text_content(Some(&default_note_name));
    } else {
        dev_log!("Warning: tonal-center-display element not found in HTML");
    }

    if let Some(volume_display) = document.get_element_by_id("tonal-center-volume-display") {
        volume_display.set_text_content(Some(&slider_position_to_db_display(0.0)));
    } else {
        dev_log!("Warning: tonal-center-volume-display element not found in HTML");
    }

    if let Some(volume_slider) = document.get_element_by_id("tonal-center-volume") {
        if let Some(html_slider) = volume_slider.dyn_ref::<HtmlInputElement>() {
            html_slider.set_value("0");
        }
    } else {
        dev_log!("Warning: tonal-center-volume element not found in HTML");
    }

    // Initialize volume icon state
    update_volume_icon_state(true);

    // Set initial display range from stored value
    if let Ok(current) = CURRENT_DISPLAY_RANGE.try_lock() {
        let id = match *current {
            DisplayRange::TwoOctaves => "display-range-two-octaves",
            DisplayRange::OneFullOctave => "display-range-one-octave",
            DisplayRange::TwoHalfOctaves => "display-range-two-half-octaves",
        };

        // Find and check the appropriate radio button by ID
        if let Some(radio_button) = document.get_element_by_id(id) {
            if let Some(input) = radio_button.dyn_ref::<HtmlInputElement>() {
                input.set_checked(true);
            }
        }
    }

    // Verify essential elements exist
    if document.get_element_by_id("tonal-center-plus").is_none() {
        dev_log!("Warning: tonal-center-plus element not found in HTML");
    }
    if document.get_element_by_id("tonal-center-minus").is_none() {
        dev_log!("Warning: tonal-center-minus element not found in HTML");
    }
    if document.get_element_by_id("tuning-system-select").is_none() {
        dev_log!("Warning: tuning-system-select element not found in HTML");
    }
    if document.get_element_by_id("scale-select").is_none() {
        dev_log!("Warning: scale-select element not found in HTML");
    }
    if document.get_element_by_id("volume-icon").is_none() {
        dev_log!("Warning: volume-icon element not found in HTML");
    }
}

pub fn cleanup_sidebar_controls() {
}

fn add_event_listener<F>(element_id: &str, event_type: &str, handler: F) 
where 
    F: FnMut(web_sys::Event) + 'static,
{
    let Some(window) = window() else { return; };
    let Some(document) = window.document() else { return; };
    let Some(element) = document.get_element_by_id(element_id) else {
        dev_log!("Failed to find {} element", element_id);
        return;
    };
    let Some(event_target) = element.dyn_ref::<EventTarget>() else { return; };
    
    let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
    if let Err(_e) = event_target.add_event_listener_with_callback(event_type, closure.as_ref().unchecked_ref()) {
        dev_log!("Failed to add {} listener to {}: {:?}", event_type, element_id, _e);
    }
    closure.forget();
}

pub fn setup_event_listeners(presenter: Rc<RefCell<crate::presentation::Presenter>>) {
    let presenter_clone = presenter.clone();
    add_event_listener("volume-icon", "click", move |_event: web_sys::Event| {
        let Some(window) = web_sys::window() else { return; };
        let Some(document) = window.document() else { return; };
        let Some(slider_element) = document.get_element_by_id("tonal-center-volume") else { return; };
        let Some(html_slider) = slider_element.dyn_ref::<HtmlInputElement>() else { return; };

        let current_position = CURRENT_TONAL_CENTER_VOLUME_POSITION.load(Ordering::Relaxed);
        let new_position = if current_position == 0 {
            // Unmute: restore remembered volume
            let remembered = REMEMBERED_VOLUME_POSITION.load(Ordering::Relaxed);
            html_slider.set_value(&remembered.to_string());
            CURRENT_TONAL_CENTER_VOLUME_POSITION.store(remembered, Ordering::Relaxed);
            update_volume_icon_state(false);
            remembered
        } else {
            // Mute: save current volume and set to 0
            REMEMBERED_VOLUME_POSITION.store(current_position, Ordering::Relaxed);
            html_slider.set_value("0");
            CURRENT_TONAL_CENTER_VOLUME_POSITION.store(0, Ordering::Relaxed);
            update_volume_icon_state(true);
            0
        };

        // Update volume display
        if let Some(display_element) = document.get_element_by_id("tonal-center-volume-display") {
            display_element.set_text_content(Some(&slider_position_to_db_display(new_position as f32)));
        }

        // Notify presenter
        let amplitude = slider_position_to_amplitude(new_position as f32);
        let current_tonal_center = CURRENT_TONAL_CENTER_NOTE.load(Ordering::Relaxed);
        presenter_clone.borrow_mut().on_tonal_center_configured(true, current_tonal_center, amplitude);
    });

    let presenter_clone = presenter.clone();
    add_event_listener("tonal-center-plus", "click", move |_event: web_sys::Event| {
        let current_tonal_center_note = CURRENT_TONAL_CENTER_NOTE.load(Ordering::Relaxed);
        if let Some(new_tonal_center_note) = increment_midi_note(current_tonal_center_note) {
            if let Ok(mut presenter_mut) = presenter_clone.try_borrow_mut() {
                let position = CURRENT_TONAL_CENTER_VOLUME_POSITION.load(Ordering::Relaxed) as f32;
                let amplitude = slider_position_to_amplitude(position);
                presenter_mut.on_tonal_center_configured(true, new_tonal_center_note, amplitude);
            }
        }
    });

    let presenter_clone = presenter.clone();
    add_event_listener("tonal-center-minus", "click", move |_event: web_sys::Event| {
        let current_tonal_center_note = CURRENT_TONAL_CENTER_NOTE.load(Ordering::Relaxed);
        if let Some(new_tonal_center_note) = decrement_midi_note(current_tonal_center_note) {
            if let Ok(mut presenter_mut) = presenter_clone.try_borrow_mut() {
                let position = CURRENT_TONAL_CENTER_VOLUME_POSITION.load(Ordering::Relaxed) as f32;
                let amplitude = slider_position_to_amplitude(position);
                presenter_mut.on_tonal_center_configured(true, new_tonal_center_note, amplitude);
            }
        }
    });

    let presenter_clone = presenter.clone();
    add_event_listener("tuning-system-select", "change", move |_event: web_sys::Event| {
        let Some(window) = web_sys::window() else { return; };
        let Some(document) = window.document() else { return; };
        let Some(select_element) = document.get_element_by_id("tuning-system-select") else { return; };
        let Some(html_select) = select_element.dyn_ref::<HtmlSelectElement>() else { return; };
        
        let tuning_system = match html_select.value().as_str() {
            "equal" => TuningSystem::EqualTemperament,
            "just" => TuningSystem::JustIntonation,
            _ => {
                dev_log!("Unknown tuning system value: {}", html_select.value());
                return;
            }
        };
        presenter_clone.borrow_mut().on_tuning_system_changed(tuning_system);
    });

    let presenter_clone = presenter.clone();
    add_event_listener("scale-select", "change", move |_event: web_sys::Event| {
        let Some(window) = web_sys::window() else { return; };
        let Some(document) = window.document() else { return; };
        let Some(select_element) = document.get_element_by_id("scale-select") else { return; };
        let Some(html_select) = select_element.dyn_ref::<HtmlSelectElement>() else { return; };
        
        let scale = match html_select.value().as_str() {
            "chromatic" => Scale::Chromatic,
            "major" => Scale::Major,
            "minor" => Scale::Minor,
            "harmonic_minor" => Scale::HarmonicMinor,
            "melodic_minor" => Scale::MelodicMinor,
            "major_pentatonic" => Scale::MajorPentatonic,
            "minor_pentatonic" => Scale::MinorPentatonic,
            "blues" => Scale::Blues,
            "dorian" => Scale::Dorian,
            "phrygian" => Scale::Phrygian,
            "lydian" => Scale::Lydian,
            "mixolydian" => Scale::Mixolydian,
            "locrian" => Scale::Locrian,
            "whole_tone" => Scale::WholeTone,
            "augmented" => Scale::Augmented,
            "diminished_half_whole" => Scale::DiminishedHalfWhole,
            "diminished_whole_half" => Scale::DiminishedWholeHalf,
            "hungarian_minor" => Scale::HungarianMinor,
            "neapolitan_minor" => Scale::NeapolitanMinor,
            "neapolitan_major" => Scale::NeapolitanMajor,
            "enigmatic" => Scale::Enigmatic,
            "persian" => Scale::Persian,
            "double_harmonic_major" => Scale::DoubleHarmonicMajor,
            "altered" => Scale::Altered,
            "bebop_major" => Scale::BebopMajor,
            "bebop_dominant" => Scale::BebopDominant,
            _ => {
                dev_log!("Unknown scale value: {}", html_select.value());
                return;
            }
        };
        presenter_clone.borrow_mut().on_scale_changed(scale);
    });

    // Add event listeners for display range radio buttons
    let presenter_clone_1 = presenter.clone();
    add_event_listener("display-range-two-octaves", "change", move |_event: web_sys::Event| {
        let display_range = DisplayRange::TwoOctaves;
        if let Ok(mut current) = CURRENT_DISPLAY_RANGE.try_lock() {
            *current = display_range.clone();
        }
        presenter_clone_1.borrow_mut().on_display_range_changed(display_range);
    });

    let presenter_clone_2 = presenter.clone();
    add_event_listener("display-range-one-octave", "change", move |_event: web_sys::Event| {
        let display_range = DisplayRange::OneFullOctave;
        if let Ok(mut current) = CURRENT_DISPLAY_RANGE.try_lock() {
            *current = display_range.clone();
        }
        presenter_clone_2.borrow_mut().on_display_range_changed(display_range);
    });

    let presenter_clone_3 = presenter.clone();
    add_event_listener("display-range-two-half-octaves", "change", move |_event: web_sys::Event| {
        let display_range = DisplayRange::TwoHalfOctaves;
        if let Ok(mut current) = CURRENT_DISPLAY_RANGE.try_lock() {
            *current = display_range.clone();
        }
        presenter_clone_3.borrow_mut().on_display_range_changed(display_range);
    });

    let presenter_clone = presenter.clone();
    add_event_listener("tonal-center-volume", "input", move |_event: web_sys::Event| {
        let Some(window) = web_sys::window() else { return; };
        let Some(document) = window.document() else { return; };
        let Some(slider_element) = document.get_element_by_id("tonal-center-volume") else { return; };
        let Some(html_slider) = slider_element.dyn_ref::<HtmlInputElement>() else { return; };
        let Ok(position) = html_slider.value().parse::<f32>() else { return; };
        
        CURRENT_TONAL_CENTER_VOLUME_POSITION.store(position as u8, Ordering::Relaxed);
        let amplitude = slider_position_to_amplitude(position);

        // Update icon state based on position
        update_volume_icon_state(position == 0.0);

        // If moving from 0, reset remembered volume to default
        if position > 0.0 {
            let previous_position = CURRENT_TONAL_CENTER_VOLUME_POSITION.swap(position as u8, Ordering::Relaxed);
            if previous_position == 0 {
                REMEMBERED_VOLUME_POSITION.store(DEFAULT_VOLUME_POSITION, Ordering::Relaxed);
            }
        }

        if let Some(display_element) = document.get_element_by_id("tonal-center-volume-display") {
            display_element.set_text_content(Some(&slider_position_to_db_display(position)));
        }
        
        let current_tonal_center = CURRENT_TONAL_CENTER_NOTE.load(Ordering::Relaxed);
        presenter_clone.borrow_mut().on_tonal_center_configured(true, current_tonal_center, amplitude);
    });
}


pub fn sync_sidebar_with_presenter_state(model_data: &crate::common::shared_types::ModelUpdateResult) {
    let Some(window) = window() else {
        return;
    };

    let Some(document) = window.document() else {
        return;
    };

    CURRENT_TONAL_CENTER_NOTE.store(model_data.tonal_center_note, Ordering::Relaxed);

    // Get the current display range
    let display_range = if let Ok(current) = CURRENT_DISPLAY_RANGE.try_lock() {
        current.clone()
    } else {
        crate::app_config::DEFAULT_DISPLAY_RANGE
    };

    // Save configuration to local storage only if it changed
    let current_config = (model_data.tonal_center_note, model_data.tuning_system, model_data.scale, display_range.clone());
    if let Ok(mut last_saved) = LAST_SAVED_CONFIG.try_lock() {
        if last_saved.as_ref() != Some(&current_config) {
            storage::save_config(
                model_data.tonal_center_note,
                model_data.tuning_system,
                model_data.scale,
                display_range
            );
            *last_saved = Some(current_config);
        }
    }

    if let Some(display) = document.get_element_by_id("tonal-center-display") {
        let formatted_note = crate::common::shared_types::midi_note_to_name(model_data.tonal_center_note);
        display.set_text_content(Some(&formatted_note));
    }
    if let Some(select_element) = document.get_element_by_id("tuning-system-select") {
        if let Some(html_select) = select_element.dyn_ref::<HtmlSelectElement>() {
            let value = match model_data.tuning_system {
                TuningSystem::EqualTemperament => "equal",
                TuningSystem::JustIntonation => "just",
            };
            html_select.set_value(value);
        }
    }
    if let Some(select_element) = document.get_element_by_id("scale-select") {
        if let Some(html_select) = select_element.dyn_ref::<HtmlSelectElement>() {
            let value = match model_data.scale {
                Scale::Chromatic => "chromatic",
                Scale::Major => "major",
                Scale::Minor => "minor",
                Scale::HarmonicMinor => "harmonic_minor",
                Scale::MelodicMinor => "melodic_minor",
                Scale::MajorPentatonic => "major_pentatonic",
                Scale::MinorPentatonic => "minor_pentatonic",
                Scale::Blues => "blues",
                Scale::Dorian => "dorian",
                Scale::Phrygian => "phrygian",
                Scale::Lydian => "lydian",
                Scale::Mixolydian => "mixolydian",
                Scale::Locrian => "locrian",
                Scale::WholeTone => "whole_tone",
                Scale::Augmented => "augmented",
                Scale::DiminishedHalfWhole => "diminished_half_whole",
                Scale::DiminishedWholeHalf => "diminished_whole_half",
                Scale::HungarianMinor => "hungarian_minor",
                Scale::NeapolitanMinor => "neapolitan_minor",
                Scale::NeapolitanMajor => "neapolitan_major",
                Scale::Enigmatic => "enigmatic",
                Scale::Persian => "persian",
                Scale::DoubleHarmonicMajor => "double_harmonic_major",
                Scale::Altered => "altered",
                Scale::BebopMajor => "bebop_major",
                Scale::BebopDominant => "bebop_dominant",
            };
            html_select.set_value(value);
        }
    }
    let current_position = CURRENT_TONAL_CENTER_VOLUME_POSITION.load(Ordering::Relaxed) as f32;
    if let Some(slider_element) = document.get_element_by_id("tonal-center-volume") {
        if let Some(html_slider) = slider_element.dyn_ref::<HtmlInputElement>() {
            html_slider.set_value(&current_position.to_string());
        }
    }
    if let Some(display_element) = document.get_element_by_id("tonal-center-volume-display") {
        display_element.set_text_content(Some(&slider_position_to_db_display(current_position)));
    }

}

