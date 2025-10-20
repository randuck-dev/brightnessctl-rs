use brightnessctl_rs::read_all_brightness_devices;
use gtk4::gdk::Display;
use gtk4::glib::{self, IOCondition, SourceId};
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, CssProvider, Label, Orientation, ProgressBar};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::os::unix::io::AsRawFd;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::Duration;
use udev::MonitorBuilder;

pub fn main() {
    let app = Application::builder()
        .application_id("com.randuck-dev.brightnessctl-rs-client")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn get_current_brightness() -> String {
    let devices = read_all_brightness_devices();
    if let Some(device) = devices.first() {
        format!("Brightness: {}", device.brightness.0)
    } else {
        "No device found".to_string()
    }
}

fn get_brightness_percentage() -> f64 {
    let devices = read_all_brightness_devices();
    if let Some(device) = devices.first() {
        device.brightness.0 as f64 / device.max_brightness.0 as f64
    } else {
        0.0
    }
}

fn build_ui(app: &Application) {
    // Load CSS for rounded corners and progress bar styling
    let css_provider = CssProvider::new();
    css_provider.load_from_data(
        "window {
            background-color: rgba(0, 0, 0, 0.8);
            border-radius: 10px;
        }
        label {
            color: white;
            font-size: 16px;
        }
        progressbar {
            min-height: 8px;
            min-width: 200px;
        }
        progressbar trough {
            background-color: rgba(255, 255, 255, 0.2);
            border-radius: 4px;
        }
        progressbar progress {
            background-color: white;
            border-radius: 4px;
        }",
    );

    gtk4::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to display"),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let window = ApplicationWindow::builder().application(app).build();

    // Initialize layer shell
    window.init_layer_shell();

    // Set as overlay layer
    window.set_layer(Layer::Overlay);

    // Anchor to bottom center
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, false);
    window.set_anchor(Edge::Right, false);
    window.set_anchor(Edge::Top, false);

    // Set margins
    window.set_margin(Edge::Bottom, 20);

    // Create a vertical box to hold label and progress bar
    let vbox = Box::new(Orientation::Vertical, 10);
    vbox.set_margin_start(20);
    vbox.set_margin_end(20);
    vbox.set_margin_top(10);
    vbox.set_margin_bottom(10);

    // Create label with current brightness
    let label = Label::new(Some(&get_current_brightness()));

    // Create progress bar
    let progress_bar = ProgressBar::new();
    progress_bar.set_fraction(get_brightness_percentage());

    vbox.append(&label);
    vbox.append(&progress_bar);

    window.set_child(Some(&vbox));

    // Start hidden
    window.set_visible(false);

    // Set up udev monitor for backlight changes
    let monitor = MonitorBuilder::new()
        .expect("Failed to create udev monitor")
        .match_subsystem("backlight")
        .expect("Failed to match backlight subsystem")
        .listen()
        .expect("Failed to listen to udev events");

    let socket = Rc::new(Mutex::new(monitor));
    let label_clone = label.clone();
    let progress_bar_clone = progress_bar.clone();
    let window_clone = window.clone();

    // Store the timeout source ID so we can cancel it
    let timeout_id: Rc<RefCell<Option<SourceId>>> = Rc::new(RefCell::new(None));
    let timeout_id_clone = timeout_id.clone();

    let fd = socket.lock().unwrap().as_raw_fd();

    gtk4::glib::unix_fd_add_local(fd, IOCondition::IN, move |_fd, _condition| {
        // Read the udev event
        if let Ok(socket_guard) = socket.lock() {
            if let Some(event) = socket_guard.iter().next() {
                println!("Backlight event: {:?}", event.event_type());

                // Update the label with new brightness value
                let new_brightness = get_current_brightness();
                label_clone.set_text(&new_brightness);

                // Update the progress bar
                let percentage = get_brightness_percentage();
                progress_bar_clone.set_fraction(percentage);

                // Show the window
                window_clone.set_visible(true);

                // Cancel existing timeout if any
                if let Some(id) = timeout_id_clone.borrow_mut().take() {
                    id.remove();
                }

                // Set up new timeout to hide after 5 seconds
                let window_timeout = window_clone.clone();
                let timeout_id_for_clear = timeout_id_clone.clone();
                let new_timeout_id = glib::timeout_add_local(Duration::from_secs(5), move || {
                    window_timeout.set_visible(false);
                    // Clear the timeout ID since it has completed
                    *timeout_id_for_clear.borrow_mut() = None;
                    glib::ControlFlow::Break
                });

                *timeout_id_clone.borrow_mut() = Some(new_timeout_id);
            }
        }

        gtk4::glib::ControlFlow::Continue
    });
}
