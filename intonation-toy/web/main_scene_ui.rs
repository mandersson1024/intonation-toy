#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use web_sys::{window, Document, HtmlElement, HtmlSelectElement, HtmlInputElement, EventTarget};

#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::sync::atomic::{AtomicU8, AtomicI8, Ordering};

#[cfg(target_arch = "wasm32")]
use crate::common::dev_log;
#[cfg(target_arch = "wasm32")]
use crate::shared_types::{TuningSystem, MidiNote, Scale, increment_midi_note, decrement_midi_note};
#[cfg(target_arch = "wasm32")]
use crate::web::styling;

// Global state for current root note - initialized to A3 (57)
#[cfg(target_arch = "wasm32")]
static CURRENT_ROOT_NOTE: AtomicU8 = AtomicU8::new(57);


// Global state for tuning fork volume slider position (0-100)
#[cfg(target_arch = "wasm32")]
static CURRENT_TUNING_FORK_VOLUME_POSITION: AtomicU8 = AtomicU8::new(0);

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

    // No need for local style constants - will use centralized styling functions

    // Create header for sidebar
    let Ok(header) = document.create_element("h1") else {
        dev_log!("Failed to create header");
        return;
    };
    header.set_text_content(Some("Intonation Toy"));
    header.set_attribute("class", "sidebar-header").ok();

    // Create container div
    let Ok(container) = document.create_element("div") else {
        dev_log!("Failed to create container div");
        return;
    };
    
    container.set_id("main-scene-ui-container");
    
    container.set_attribute("class", "ui-container").ok();

    // Create root note section header
    let Ok(root_note_header) = document.create_element("div") else {
        dev_log!("Failed to create root note header");
        return;
    };
    root_note_header.set_text_content(Some("Root Note"));
    root_note_header.set_attribute("class", "subsection-header").ok();

    // Create root note controls container
    let Ok(root_note_container) = document.create_element("div") else {
        dev_log!("Failed to create root note container");
        return;
    };
    root_note_container.set_attribute("style", "display: flex; align-items: center; gap: 8px;").ok();

    // Create minus button
    let Ok(minus_button) = document.create_element("button") else {
        dev_log!("Failed to create minus button");
        return;
    };
    minus_button.set_id("root-note-minus");
    minus_button.set_text_content(Some("-"));
    minus_button.set_attribute("class", "small-button").ok();

    // Create root note display
    let Ok(root_note_display) = document.create_element("span") else {
        dev_log!("Failed to create root note display");
        return;
    };
    root_note_display.set_id("root-note-display");
    root_note_display.set_text_content(Some("A3")); // Default root note is 57 (A3)
    root_note_display.set_attribute("class", "root-note-display").ok();

    // Create plus button
    let Ok(plus_button) = document.create_element("button") else {
        dev_log!("Failed to create plus button");
        return;
    };
    plus_button.set_id("root-note-plus");
    plus_button.set_text_content(Some("+"));
    plus_button.set_attribute("class", "small-button").ok();

    // Assemble root note controls
    root_note_container.append_child(&minus_button).ok();
    root_note_container.append_child(&root_note_display).ok();
    root_note_container.append_child(&plus_button).ok();

    // Create tuning section header
    let Ok(tuning_header) = document.create_element("div") else {
        dev_log!("Failed to create tuning header");
        return;
    };
    tuning_header.set_text_content(Some("Tuning System"));
    tuning_header.set_attribute("class", "subsection-header").ok();

    // Create tuning system controls container
    let Ok(tuning_container) = document.create_element("div") else {
        dev_log!("Failed to create tuning container");
        return;
    };
    tuning_container.set_attribute("style", "display: flex; align-items: center; gap: 8px;").ok();

    // Create tuning system dropdown
    let Ok(tuning_select) = document.create_element("select") else {
        dev_log!("Failed to create tuning select");
        return;
    };
    tuning_select.set_id("tuning-system-select");
    tuning_select.set_attribute("class", "control-select").ok();

    // Create Equal Temperament option
    let Ok(equal_option) = document.create_element("option") else {
        dev_log!("Failed to create equal temperament option");
        return;
    };
    equal_option.set_attribute("value", "equal").ok();
    equal_option.set_text_content(Some("Equal Temperament"));

    // Create Just Intonation option
    let Ok(just_option) = document.create_element("option") else {
        dev_log!("Failed to create just intonation option");
        return;
    };
    just_option.set_attribute("value", "just").ok();
    just_option.set_text_content(Some("Just Intonation"));

    // Assemble tuning system dropdown
    tuning_select.append_child(&equal_option).ok();
    tuning_select.append_child(&just_option).ok();

    // Assemble tuning controls
    tuning_container.append_child(&tuning_select).ok();

    // Create scale section header
    let Ok(scale_header) = document.create_element("div") else {
        dev_log!("Failed to create scale header");
        return;
    };
    scale_header.set_text_content(Some("Scale"));
    scale_header.set_attribute("class", "subsection-header").ok();

    // Create scale controls container
    let Ok(scale_container) = document.create_element("div") else {
        dev_log!("Failed to create scale container");
        return;
    };
    scale_container.set_attribute("style", "display: flex; align-items: center; gap: 8px;").ok();

    // Create scale dropdown
    let Ok(scale_select) = document.create_element("select") else {
        dev_log!("Failed to create scale select");
        return;
    };
    scale_select.set_id("scale-select");
    scale_select.set_attribute("class", "control-select").ok();

    // Create Chromatic option (first in enum, so default)
    let Ok(chromatic_option) = document.create_element("option") else {
        dev_log!("Failed to create chromatic option");
        return;
    };
    chromatic_option.set_attribute("value", "chromatic").ok();
    chromatic_option.set_attribute("selected", "true").ok();
    chromatic_option.set_text_content(Some("Chromatic"));

    // Create Major option
    let Ok(major_option) = document.create_element("option") else {
        dev_log!("Failed to create major option");
        return;
    };
    major_option.set_attribute("value", "major").ok();
    major_option.set_text_content(Some("Major"));

    // Create Minor option
    let Ok(minor_option) = document.create_element("option") else {
        dev_log!("Failed to create minor option");
        return;
    };
    minor_option.set_attribute("value", "minor").ok();
    minor_option.set_text_content(Some("Minor"));

    // Create Major Pentatonic option
    let Ok(major_pentatonic_option) = document.create_element("option") else {
        dev_log!("Failed to create major pentatonic option");
        return;
    };
    major_pentatonic_option.set_attribute("value", "major_pentatonic").ok();
    major_pentatonic_option.set_text_content(Some("Major Pentatonic"));

    // Create Minor Pentatonic option
    let Ok(minor_pentatonic_option) = document.create_element("option") else {
        dev_log!("Failed to create minor pentatonic option");
        return;
    };
    minor_pentatonic_option.set_attribute("value", "minor_pentatonic").ok();
    minor_pentatonic_option.set_text_content(Some("Minor Pentatonic"));

    // Assemble scale dropdown (order matches enum)
    scale_select.append_child(&chromatic_option).ok();
    scale_select.append_child(&major_option).ok();
    scale_select.append_child(&minor_option).ok();
    scale_select.append_child(&major_pentatonic_option).ok();
    scale_select.append_child(&minor_pentatonic_option).ok();

    // Assemble scale controls
    scale_container.append_child(&scale_select).ok();

    // Create Tuning Fork section header
    let Ok(tuning_fork_header) = document.create_element("div") else {
        dev_log!("Failed to create tuning fork header");
        return;
    };
    tuning_fork_header.set_text_content(Some("Tuning Fork"));
    tuning_fork_header.set_attribute("class", "subsection-header").ok();

    // Create tuning fork container
    let Ok(tuning_fork_container) = document.create_element("div") else {
        dev_log!("Failed to create tuning fork container");
        return;
    };
    tuning_fork_container.set_attribute("class", "tuning-fork-container").ok();


    // Create volume container
    let Ok(volume_container) = document.create_element("div") else {
        dev_log!("Failed to create volume container");
        return;
    };
    volume_container.set_attribute("style", "display: flex; flex-direction: column; gap: 4px;").ok();

    // Create volume label
    let Ok(volume_label) = document.create_element("label") else {
        dev_log!("Failed to create volume label");
        return;
    };
    volume_label.set_attribute("for", "tuning-fork-volume").ok();
    volume_label.set_text_content(Some("Volume (dB)"));
    volume_label.set_attribute("style", "color: var(--text-color); font-size: 14px;").ok();

    // Create volume slider
    let Ok(volume_slider) = document.create_element("input") else {
        dev_log!("Failed to create volume slider");
        return;
    };
    volume_slider.set_id("tuning-fork-volume");
    volume_slider.set_attribute("type", "range").ok();
    volume_slider.set_attribute("min", "0").ok();
    volume_slider.set_attribute("max", "100").ok();
    volume_slider.set_attribute("value", "0").ok();
    // Range input styling is applied globally via apply_range_input_styles()
    // No need for individual styling

    // Create volume display
    let Ok(volume_display) = document.create_element("span") else {
        dev_log!("Failed to create volume display");
        return;
    };
    volume_display.set_id("tuning-fork-volume-display");
    volume_display.set_text_content(Some(&slider_position_to_db_display(0.0)));
    volume_display.set_attribute("class", "volume-display").ok();

    // Assemble volume container
    volume_container.append_child(&volume_label).ok();
    volume_container.append_child(&volume_slider).ok();
    volume_container.append_child(&volume_display).ok();

    // Assemble tuning fork controls
    tuning_fork_container.append_child(&volume_container).ok();

    // Create About section
    let Ok(about_section) = document.create_element("div") else {
        dev_log!("Failed to create about section");
        return;
    };
    about_section.set_attribute("class", "about-section").ok();

    // Create About header
    let Ok(about_header) = document.create_element("div") else {
        dev_log!("Failed to create about header");
        return;
    };
    about_header.set_attribute("class", "about-header").ok();
    about_header.set_text_content(Some("About"));

    // Create About content
    let Ok(about_content) = document.create_element("div") else {
        dev_log!("Failed to create about content");
        return;
    };
    about_content.set_attribute("class", "about-content").ok();

    // Create app description
    let Ok(app_description) = document.create_element("p") else {
        dev_log!("Failed to create app description");
        return;
    };
    app_description.set_attribute("class", "about-text").ok();
    app_description.set_inner_html("<strong>Intonation Toy</strong> is a real-time pitch analysis and visualization tool. It helps you explore musical intonation by analyzing audio input and displaying the relationship between detected pitches and your selected root note.");

    // Create user guide section
    let Ok(user_guide_header) = document.create_element("h3") else {
        dev_log!("Failed to create user guide header");
        return;
    };
    user_guide_header.set_text_content(Some("Quick Guide"));

    let Ok(user_guide) = document.create_element("ul") else {
        dev_log!("Failed to create user guide");
        return;
    };
    user_guide.set_attribute("class", "about-list").ok();
    user_guide.set_inner_html(r#"
        <li><strong>Root Note:</strong> Use +/- buttons to adjust the tonic pitch</li>
        <li><strong>Tuning Fork:</strong> Adjust volume of the root note reference tone with slider</li>
        <li><strong>Tuning System:</strong> Choose between Equal Temperament or Just Intonation</li>
        <li><strong>Scale:</strong> Select the musical scale for pitch visualization</li>
        <li><strong>Microphone:</strong> Grant permission when prompted to analyze live audio</li>
    "#);

    // Create browser requirements section
    let Ok(browser_header) = document.create_element("h3") else {
        dev_log!("Failed to create browser header");
        return;
    };
    browser_header.set_text_content(Some("Browser Requirements"));

    let Ok(browser_info) = document.create_element("p") else {
        dev_log!("Failed to create browser info");
        return;
    };
    browser_info.set_attribute("class", "about-text").ok();
    browser_info.set_text_content(Some("Works best in modern browsers with WebAssembly and Web Audio API support. Chrome, Firefox, Safari, and Edge are recommended."));

    // Assemble About content
    about_content.append_child(&app_description).ok();
    about_content.append_child(&user_guide_header).ok();
    about_content.append_child(&user_guide).ok();
    about_content.append_child(&browser_header).ok();
    about_content.append_child(&browser_info).ok();

    // Assemble About section
    about_section.append_child(&about_header).ok();
    about_section.append_child(&about_content).ok();

    // Assemble main container
    container.append_child(&root_note_header).ok();
    container.append_child(&root_note_container).ok();
    container.append_child(&tuning_header).ok();
    container.append_child(&tuning_container).ok();
    container.append_child(&scale_header).ok();
    container.append_child(&scale_container).ok();
    container.append_child(&tuning_fork_header).ok();
    container.append_child(&tuning_fork_container).ok();
    container.append_child(&about_section).ok();


    // Append header and container to sidebar
    let sidebar = document.get_element_by_id("sidebar");
    
    if let Some(sidebar) = sidebar {
        // Append header first
        if let Err(err) = sidebar.append_child(&header) {
            dev_log!("Failed to append header to sidebar: {:?}", err);
        }
        // Then append the main container
        if let Err(err) = sidebar.append_child(&container) {
            dev_log!("Failed to append UI container to sidebar: {:?}", err);
        }
    } else {
        // Fallback to body if sidebar doesn't exist
        if let Some(body) = document.body() {
            if let Err(err) = body.append_child(&header) {
                dev_log!("Failed to append header to body: {:?}", err);
            }
            if let Err(err) = body.append_child(&container) {
                dev_log!("Failed to append UI container to body: {:?}", err);
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn cleanup_main_scene_ui() {
    let Some(window) = window() else {
        return;
    };

    let Some(document) = window.document() else {
        return;
    };

    // Remove the main container
    if let Some(container) = document.get_element_by_id("main-scene-ui-container") {
        container.remove();
    }
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
}


#[cfg(not(target_arch = "wasm32"))]
pub fn setup_event_listeners(_presenter: std::rc::Rc<std::cell::RefCell<crate::presentation::Presenter>>) {
    // No-op for non-WASM targets
}

#[cfg(not(target_arch = "wasm32"))]
pub fn sync_ui_with_presenter_state(_model_data: &crate::shared_types::ModelUpdateResult) {
    // No-op for non-WASM targets
}