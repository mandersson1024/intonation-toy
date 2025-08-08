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

// Global state for root note audio enabled
#[cfg(target_arch = "wasm32")]
static CURRENT_ROOT_NOTE_AUDIO_ENABLED: AtomicU8 = AtomicU8::new(0); // 0 = false, 1 = true

// Debouncing for tuning fork clicks to prevent double-triggering
#[cfg(target_arch = "wasm32")]
static LAST_TUNING_FORK_CLICK_TIME: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

// Global state for tuning fork volume
#[cfg(target_arch = "wasm32")]
static CURRENT_TUNING_FORK_VOLUME_DB: AtomicI8 = AtomicI8::new(-20);

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
    header.set_attribute("style", &styling::get_sidebar_header_style()).ok();

    // Create container div
    let Ok(container) = document.create_element("div") else {
        dev_log!("Failed to create container div");
        return;
    };
    
    container.set_id("main-scene-ui-container");
    
    container.set_attribute("style", &styling::get_container_style()).ok();

    // Create root note section header
    let Ok(root_note_header) = document.create_element("div") else {
        dev_log!("Failed to create root note header");
        return;
    };
    root_note_header.set_text_content(Some("Root Note"));
    root_note_header.set_attribute("style", &styling::get_subsection_header_style()).ok();

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
    minus_button.set_attribute("style", &styling::get_small_button_style()).ok();

    // Create root note display
    let Ok(root_note_display) = document.create_element("span") else {
        dev_log!("Failed to create root note display");
        return;
    };
    root_note_display.set_id("root-note-display");
    root_note_display.set_text_content(Some("A3")); // Default root note is 57 (A3)
    root_note_display.set_attribute("style", &styling::get_root_note_display_style()).ok();

    // Create plus button
    let Ok(plus_button) = document.create_element("button") else {
        dev_log!("Failed to create plus button");
        return;
    };
    plus_button.set_id("root-note-plus");
    plus_button.set_text_content(Some("+"));
    plus_button.set_attribute("style", &styling::get_small_button_style()).ok();

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
    tuning_header.set_attribute("style", &styling::get_subsection_header_style()).ok();

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
    tuning_select.set_attribute("style", &styling::get_select_style()).ok();

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
    scale_header.set_attribute("style", &styling::get_subsection_header_style()).ok();

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
    scale_select.set_attribute("style", &styling::get_select_style()).ok();

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
    tuning_fork_header.set_attribute("style", &styling::get_subsection_header_style()).ok();

    // Create tuning fork container
    let Ok(tuning_fork_container) = document.create_element("div") else {
        dev_log!("Failed to create tuning fork container");
        return;
    };
    tuning_fork_container.set_attribute("style", "display: flex; flex-direction: column; gap: 8px;").ok();

    // Create enable checkbox container
    let Ok(checkbox_container) = document.create_element("div") else {
        dev_log!("Failed to create checkbox container");
        return;
    };
    checkbox_container.set_attribute("style", "display: flex; align-items: center; gap: 8px;").ok();

    // Create enable checkbox
    let Ok(enable_checkbox) = document.create_element("input") else {
        dev_log!("Failed to create enable checkbox");
        return;
    };
    enable_checkbox.set_id("tuning-fork-enable");
    enable_checkbox.set_attribute("type", "checkbox").ok();
    enable_checkbox.set_attribute("style", &styling::get_checkbox_style()).ok();

    // Create enable label
    let Ok(enable_label) = document.create_element("label") else {
        dev_log!("Failed to create enable label");
        return;
    };
    enable_label.set_attribute("for", "tuning-fork-enable").ok();
    enable_label.set_text_content(Some("Enable"));
    enable_label.set_attribute("style", "color: var(--text-color); font-size: 14px;").ok();

    // Assemble checkbox container
    checkbox_container.append_child(&enable_checkbox).ok();
    checkbox_container.append_child(&enable_label).ok();

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
    volume_slider.set_attribute("min", "-40").ok();
    volume_slider.set_attribute("max", "0").ok();
    volume_slider.set_attribute("value", "-20").ok();
    volume_slider.set_attribute("style", &styling::get_range_input_style()).ok();

    // Create volume display
    let Ok(volume_display) = document.create_element("span") else {
        dev_log!("Failed to create volume display");
        return;
    };
    volume_display.set_id("tuning-fork-volume-display");
    volume_display.set_text_content(Some("-20 dB"));
    volume_display.set_attribute("style", &styling::get_volume_display_style()).ok();

    // Assemble volume container
    volume_container.append_child(&volume_label).ok();
    volume_container.append_child(&volume_slider).ok();
    volume_container.append_child(&volume_display).ok();

    // Assemble tuning fork controls
    tuning_fork_container.append_child(&checkbox_container).ok();
    tuning_fork_container.append_child(&volume_container).ok();

    // Create About section
    let Ok(about_section) = document.create_element("div") else {
        dev_log!("Failed to create about section");
        return;
    };
    about_section.set_class_name("about-section");
    about_section.set_attribute("style", &styling::get_about_section_style()).ok();

    // Create About header
    let Ok(about_header) = document.create_element("div") else {
        dev_log!("Failed to create about header");
        return;
    };
    about_header.set_class_name("about-header");
    about_header.set_attribute("style", &styling::get_about_header_style()).ok();
    about_header.set_text_content(Some("About"));

    // Create About content
    let Ok(about_content) = document.create_element("div") else {
        dev_log!("Failed to create about content");
        return;
    };
    about_content.set_class_name("about-content");
    about_content.set_attribute("style", &styling::get_about_content_style()).ok();

    // Create app description
    let Ok(app_description) = document.create_element("p") else {
        dev_log!("Failed to create app description");
        return;
    };
    app_description.set_class_name("about-text");
    app_description.set_attribute("style", &styling::get_about_text_style()).ok();
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
    user_guide.set_class_name("about-list");
    user_guide.set_attribute("style", &styling::get_about_list_style()).ok();
    user_guide.set_inner_html(r#"
        <li><strong>Root Note:</strong> Use +/- buttons to adjust the tonic pitch</li>
        <li><strong>Tuning Fork:</strong> Enable to hear the root note as a reference tone, adjust volume with slider</li>
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
    browser_info.set_class_name("about-text");
    browser_info.set_attribute("style", &styling::get_about_text_style()).ok();
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
                    
                    // Also update root note audio frequency if it's currently enabled
                    if let Some(current_window) = web_sys::window() {
                        if let Some(document) = current_window.document() {
                            if let Some(checkbox_element) = document.get_element_by_id("tuning-fork-enable") {
                                if let Some(html_checkbox) = checkbox_element.dyn_ref::<HtmlInputElement>() {
                                    if html_checkbox.checked() {
                                        let volume_db = CURRENT_TUNING_FORK_VOLUME_DB.load(Ordering::Relaxed) as f32;
                                        presenter_mut.on_root_note_audio_configured(true, new_root, volume_db);
                                    }
                                }
                            }
                        }
                    }
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
                    
                    // Also update root note audio frequency if it's currently enabled
                    if let Some(current_window) = web_sys::window() {
                        if let Some(document) = current_window.document() {
                            if let Some(checkbox_element) = document.get_element_by_id("tuning-fork-enable") {
                                if let Some(html_checkbox) = checkbox_element.dyn_ref::<HtmlInputElement>() {
                                    if html_checkbox.checked() {
                                        let volume_db = CURRENT_TUNING_FORK_VOLUME_DB.load(Ordering::Relaxed) as f32;
                                        presenter_mut.on_root_note_audio_configured(true, new_root, volume_db);
                                    }
                                }
                            }
                        }
                    }
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

    // Set up tuning fork enable checkbox event listener
    if let Some(checkbox) = document.get_element_by_id("tuning-fork-enable") {
        let presenter_clone = presenter.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            if let Some(current_window) = web_sys::window() {
                if let Some(document) = current_window.document() {
                    if let Some(checkbox_element) = document.get_element_by_id("tuning-fork-enable") {
                        if let Some(html_checkbox) = checkbox_element.dyn_ref::<HtmlInputElement>() {
                            let enabled = html_checkbox.checked();
                            let current_root_note = CURRENT_ROOT_NOTE.load(Ordering::Relaxed);
                            let volume_db = CURRENT_TUNING_FORK_VOLUME_DB.load(Ordering::Relaxed) as f32;
                            presenter_clone.borrow_mut().on_root_note_audio_configured(enabled, current_root_note, volume_db);
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        if let Some(event_target) = checkbox.dyn_ref::<EventTarget>() {
            if let Err(err) = event_target.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref()) {
                dev_log!("Failed to add change listener to tuning fork checkbox: {:?}", err);
            }
        }
        closure.forget();
    } else {
        dev_log!("Failed to find tuning-fork-enable checkbox");
    }

    // Set up tuning fork volume slider event listener
    if let Some(slider) = document.get_element_by_id("tuning-fork-volume") {
        let presenter_clone = presenter.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            if let Some(current_window) = web_sys::window() {
                if let Some(document) = current_window.document() {
                    if let Some(slider_element) = document.get_element_by_id("tuning-fork-volume") {
                        if let Some(html_slider) = slider_element.dyn_ref::<HtmlInputElement>() {
                            if let Ok(volume_db) = html_slider.value().parse::<i8>() {
                                CURRENT_TUNING_FORK_VOLUME_DB.store(volume_db, Ordering::Relaxed);
                                
                                // Update volume display
                                if let Some(display_element) = document.get_element_by_id("tuning-fork-volume-display") {
                                    display_element.set_text_content(Some(&format!("{} dB", volume_db)));
                                }
                                
                                // Update audio if enabled
                                if let Some(checkbox_element) = document.get_element_by_id("tuning-fork-enable") {
                                    if let Some(html_checkbox) = checkbox_element.dyn_ref::<HtmlInputElement>() {
                                        if html_checkbox.checked() {
                                            let current_root_note = CURRENT_ROOT_NOTE.load(Ordering::Relaxed);
                                            presenter_clone.borrow_mut().on_root_note_audio_configured(true, current_root_note, volume_db as f32);
                                        }
                                    }
                                }
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

    // Update tuning fork controls state
    CURRENT_ROOT_NOTE_AUDIO_ENABLED.store(if model_data.root_note_audio_enabled { 1 } else { 0 }, Ordering::Relaxed);
    
    // Update checkbox state
    if let Some(checkbox_element) = document.get_element_by_id("tuning-fork-enable") {
        if let Some(html_checkbox) = checkbox_element.dyn_ref::<HtmlInputElement>() {
            html_checkbox.set_checked(model_data.root_note_audio_enabled);
        }
    }
    
    // Update volume slider and display
    let current_volume = CURRENT_TUNING_FORK_VOLUME_DB.load(Ordering::Relaxed);
    if let Some(slider_element) = document.get_element_by_id("tuning-fork-volume") {
        if let Some(html_slider) = slider_element.dyn_ref::<HtmlInputElement>() {
            html_slider.set_value(&current_volume.to_string());
        }
    }
    if let Some(display_element) = document.get_element_by_id("tuning-fork-volume-display") {
        display_element.set_text_content(Some(&format!("{} dB", current_volume)));
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