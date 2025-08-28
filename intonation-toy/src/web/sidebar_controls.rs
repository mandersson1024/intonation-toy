#[cfg(target_arch = "wasm32")]
use {
    wasm_bindgen::JsCast,
    wasm_bindgen::closure::Closure,
    web_sys::{window, HtmlSelectElement, HtmlInputElement, EventTarget},
    std::rc::Rc,
    std::cell::RefCell,
    std::sync::atomic::{AtomicU8, Ordering},
    crate::common::dev_log,
    crate::common::shared_types::{TuningSystem, Scale, increment_midi_note, decrement_midi_note},
};

// These statics are needed because the tuning fork controls (plus/minus buttons and volume slider)
// interact with each other in a way that requires shared state:
// 1. The plus/minus buttons need to know the current note to calculate next/previous values
// 2. When the note changes, we need to send both the new note AND the current volume to the presenter
// 3. The volume position must be preserved when the note changes
// Unlike the dropdown controls (scale/tuning system) which maintain their own state in the DOM,
// these controls need coordinated state management to work together properly.
#[cfg(target_arch = "wasm32")]
static CURRENT_TUNING_FORK_NOTE: AtomicU8 = AtomicU8::new(crate::app_config::DEFAULT_TUNING_FORK_NOTE);

#[cfg(target_arch = "wasm32")]
static CURRENT_TUNING_FORK_VOLUME_POSITION: AtomicU8 = AtomicU8::new(0);

#[cfg(target_arch = "wasm32")]
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

#[cfg(target_arch = "wasm32")]
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

#[cfg(target_arch = "wasm32")]
pub fn setup_sidebar_controls() {
    let Some(window) = window() else {
        dev_log!("Failed to get window");
        return;
    };

    let Some(document) = window.document() else {
        dev_log!("Failed to get document");
        return;
    };

    if let Some(tuning_fork_display) = document.get_element_by_id("tuning-fork-display") {
        let default_note_name = crate::common::shared_types::midi_note_to_name(crate::app_config::DEFAULT_TUNING_FORK_NOTE);
        tuning_fork_display.set_text_content(Some(&default_note_name));
    } else {
        dev_log!("Warning: tuning-fork-display element not found in HTML");
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

    // Verify essential elements exist
    if document.get_element_by_id("tuning-fork-plus").is_none() {
        dev_log!("Warning: tuning-fork-plus element not found in HTML");
    }
    if document.get_element_by_id("tuning-fork-minus").is_none() {
        dev_log!("Warning: tuning-fork-minus element not found in HTML");
    }
    if document.get_element_by_id("tuning-system-select").is_none() {
        dev_log!("Warning: tuning-system-select element not found in HTML");
    }
    if document.get_element_by_id("scale-select").is_none() {
        dev_log!("Warning: scale-select element not found in HTML");
    }
}

#[cfg(target_arch = "wasm32")]
pub fn cleanup_sidebar_controls() {
}

#[cfg(target_arch = "wasm32")]
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
    if let Err(err) = event_target.add_event_listener_with_callback(event_type, closure.as_ref().unchecked_ref()) {
        dev_log!("Failed to add {} listener to {}: {:?}", event_type, element_id, err);
    }
    closure.forget();
}

#[cfg(target_arch = "wasm32")]
pub fn setup_event_listeners(presenter: Rc<RefCell<crate::presentation::Presenter>>) {
    let presenter_clone = presenter.clone();
    add_event_listener("tuning-fork-plus", "click", move |_event: web_sys::Event| {
        let current_tuning_fork_note = CURRENT_TUNING_FORK_NOTE.load(Ordering::Relaxed);
        if let Some(new_tuning_fork_note) = increment_midi_note(current_tuning_fork_note) {
            if let Ok(mut presenter_mut) = presenter_clone.try_borrow_mut() {
                presenter_mut.on_tuning_fork_adjusted(new_tuning_fork_note);
                
                let position = CURRENT_TUNING_FORK_VOLUME_POSITION.load(Ordering::Relaxed) as f32;
                let amplitude = slider_position_to_amplitude(position);
                presenter_mut.on_tuning_fork_configured(true, new_tuning_fork_note, amplitude);
            }
        }
    });

    let presenter_clone = presenter.clone();
    add_event_listener("tuning-fork-minus", "click", move |_event: web_sys::Event| {
        let current_tuning_fork_note = CURRENT_TUNING_FORK_NOTE.load(Ordering::Relaxed);
        if let Some(new_tuning_fork_note) = decrement_midi_note(current_tuning_fork_note) {
            if let Ok(mut presenter_mut) = presenter_clone.try_borrow_mut() {
                presenter_mut.on_tuning_fork_adjusted(new_tuning_fork_note);
                
                let position = CURRENT_TUNING_FORK_VOLUME_POSITION.load(Ordering::Relaxed) as f32;
                let amplitude = slider_position_to_amplitude(position);
                presenter_mut.on_tuning_fork_configured(true, new_tuning_fork_note, amplitude);
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

    let presenter_clone = presenter.clone();
    add_event_listener("tuning-fork-volume", "input", move |_event: web_sys::Event| {
        let Some(window) = web_sys::window() else { return; };
        let Some(document) = window.document() else { return; };
        let Some(slider_element) = document.get_element_by_id("tuning-fork-volume") else { return; };
        let Some(html_slider) = slider_element.dyn_ref::<HtmlInputElement>() else { return; };
        let Ok(position) = html_slider.value().parse::<f32>() else { return; };
        
        CURRENT_TUNING_FORK_VOLUME_POSITION.store(position as u8, Ordering::Relaxed);
        let amplitude = slider_position_to_amplitude(position);
        
        if let Some(display_element) = document.get_element_by_id("tuning-fork-volume-display") {
            display_element.set_text_content(Some(&slider_position_to_db_display(position)));
        }
        
        let current_tuning_fork = CURRENT_TUNING_FORK_NOTE.load(Ordering::Relaxed);
        presenter_clone.borrow_mut().on_tuning_fork_configured(true, current_tuning_fork, amplitude);
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn setup_sidebar_controls() {
}

#[cfg(not(target_arch = "wasm32"))]
pub fn cleanup_sidebar_controls() {
}

#[cfg(target_arch = "wasm32")]
pub fn sync_sidebar_with_presenter_state(model_data: &crate::common::shared_types::ModelUpdateResult) {
    let Some(window) = window() else {
        return;
    };

    let Some(document) = window.document() else {
        return;
    };

    CURRENT_TUNING_FORK_NOTE.store(model_data.tuning_fork_note, Ordering::Relaxed);

    if let Some(display) = document.get_element_by_id("tuning-fork-display") {
        let formatted_note = crate::common::shared_types::midi_note_to_name(model_data.tuning_fork_note);
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
    let current_position = CURRENT_TUNING_FORK_VOLUME_POSITION.load(Ordering::Relaxed) as f32;
    if let Some(slider_element) = document.get_element_by_id("tuning-fork-volume") {
        if let Some(html_slider) = slider_element.dyn_ref::<HtmlInputElement>() {
            html_slider.set_value(&current_position.to_string());
        }
    }
    if let Some(display_element) = document.get_element_by_id("tuning-fork-volume-display") {
        display_element.set_text_content(Some(&slider_position_to_db_display(current_position)));
    }

}

#[cfg(not(target_arch = "wasm32"))]
pub fn setup_event_listeners(_presenter: std::rc::Rc<std::cell::RefCell<crate::presentation::Presenter>>) {
}

#[cfg(not(target_arch = "wasm32"))]
pub fn sync_ui_with_presenter_state(_model_data: &crate::common::shared_types::ModelUpdateResult) {
}