#![windows_subsystem = "windows"]

use rdev::{simulate, SimulateError, Event, EventType, Key};
use std::{thread, sync::Arc, sync::Mutex, sync::{atomic::{AtomicBool, Ordering}, mpsc::channel}, time::Duration, fs::File, io::Write, io::Read};
use gtk::{prelude::*, traits::SettingsExt};
use simplyscriptor2::*;

fn main() {
    let (sendch, recvch) = channel();
    spawn_event_listener(sendch);

    let record = Arc::new(AtomicBool::new(false));
    let run = Arc::new(AtomicBool::new(false));
    let events = Arc::new(Mutex::new(Vec::new()));
    let infinite_loop = Arc::new(AtomicBool::new(false));
    let loop_count = Arc::new(Mutex::new(1));
    let delay = Arc::new(AtomicBool::new(true));
    let halt_actions = Arc::new(AtomicBool::new(false));

    let record_ref = Arc::clone(&record);
    let run_ref = Arc::clone(&run);
    let events_ref = Arc::clone(&events);
    let halt_actions_ref = Arc::clone(&halt_actions);
    spawn_event_receiver(recvch, record_ref, run_ref, events_ref, halt_actions_ref);

    let run_ref = Arc::clone(&run);
    let events_ref = Arc::clone(&events);
    let infinite_loop_ref = Arc::clone(&infinite_loop);
    let loop_count_ref = Arc::clone(&loop_count);
    let delay_ref = Arc::clone(&delay);

    // GTK causes strange bugs in macos and until the bugs are sorted out, macos only gets a command line tool
    if cfg!(target_os = "macos") {
        event_loop(events_ref, run_ref, infinite_loop_ref, loop_count_ref, delay_ref);
    } else {
        thread::spawn(move || {
            event_loop(events_ref, run_ref, infinite_loop_ref, loop_count_ref, delay_ref);
        });
    }

    let _ = gtk::init();

    let settings = gtk::Settings::default().unwrap();
    settings.set_gtk_application_prefer_dark_theme(true);

    // Create gtk window
    let app = gtk::Application::builder()
        .application_id("com.borfus.simply-scriptor-2")
        .build();

    app.connect_activate(move |app| {
        let glade_src = include_str!("../simply-scriptor-gui.glade");
        let builder = gtk::Builder::from_string(glade_src);
        let window : gtk::Window = builder.object("window").expect("Couldn't get gtk object 'window'");

        let record_button : gtk::Button = builder.object("button_record").expect("Couldn't get gtk object 'button_record'");
        let record_ref = Arc::clone(&record);
        let events_ref = Arc::clone(&events);
        let minimize : gtk::CheckButton = builder.object("checkbox_minimize").expect("Couldn't get gtk object 'checkbox_minimize'");

        let window_ref : gtk::Window = builder.object("window").expect("Couldn't get gtk object 'window'");
        let open_button : gtk::Button = builder.object("button_open").expect("Couldn't get gtk object 'button_open'.");
        let script_file_label: gtk::Label = builder.object("label_script_file").expect("Couldn't get gtk object 'label_script_file'.");
        let halt_actions_ref = Arc::clone(&halt_actions);
        open_button.connect_clicked(move |_| {
            let dialog = gtk::FileChooserDialog::with_buttons(Some("Open Script"), Some(&window_ref), gtk::FileChooserAction::Open, &[("_Open", gtk::ResponseType::Accept), ("_Cancel", gtk::ResponseType::Cancel)]);
            let script_file_label_clone = script_file_label.clone();

            halt_actions_ref.store(true, Ordering::Relaxed);
            let halt_actions_ref_clone = Arc::clone(&halt_actions_ref);
            let events_ref_clone = Arc::clone(&events_ref);
            dialog.connect_response(move |dialog, response| {
                if response == gtk::ResponseType::Cancel {
                    dialog.emit_close();
                }

                if response == gtk::ResponseType::Accept {
                    // serialize Event vector and save to .bin file
                    let file_name = String::from(dialog.file().unwrap().basename().unwrap().to_str().unwrap());
                    let file_path = String::from(dialog.file().unwrap().parse_name().as_str());

                    let mut file = File::open(&file_path[..]).unwrap();
                    let mut buffer = Vec::<u8>::new();
                    let _result = file.read_to_end(&mut buffer).unwrap();
                    let decoded : Vec<rdev::Event> = bincode::deserialize(&buffer[..]).unwrap();

                    let mut events_ref_clone = events_ref_clone.lock().unwrap();
                    events_ref_clone.clear();
                    *events_ref_clone = decoded.to_vec();

                    // set label text to file name
                    if file_name.len() > 12 {
                        script_file_label_clone.set_text(format!("{}...", &file_name[0..12]).as_str());
                    } else {
                        script_file_label_clone.set_text(file_name.as_str());
                    }
                    dialog.emit_close();
                }

                halt_actions_ref_clone.store(false, Ordering::Relaxed);
            });

            dialog.run();
        });

        let window_ref : gtk::Window = builder.object("window").expect("Couldn't get gtk object 'window'");
        let save_button: gtk::Button = builder.object("button_save").expect("Couldn't get gtk object 'button_save'.");
        let script_file_label: gtk::Label = builder.object("label_script_file").expect("Couldn't get gtk object 'label_script_file'.");
        let events_ref = Arc::clone(&events);
        let halt_actions_ref = Arc::clone(&halt_actions);
        save_button.connect_clicked(move |_| {
            let dialog = gtk::FileChooserDialog::with_buttons(Some("Save Script"), Some(&window_ref), gtk::FileChooserAction::Save, &[("_Save", gtk::ResponseType::Accept), ("_Cancel", gtk::ResponseType::Cancel)]);
            let script_file_label_clone = script_file_label.clone();

            halt_actions_ref.store(true, Ordering::Relaxed);
            let halt_actions_ref_clone = Arc::clone(&halt_actions_ref);
            let events_ref_clone = Arc::clone(&events_ref);
            dialog.connect_response(move |dialog, response| {
                if response == gtk::ResponseType::Cancel {
                    dialog.emit_close();
                }

                if response == gtk::ResponseType::Accept {
                    // serialize Event vector and save to .bin file
                    let file_name = format!("{}.bin", dialog.file().unwrap().basename().unwrap().to_str().unwrap());
                    let file_path = format!("{}.bin", dialog.file().unwrap().parse_name().as_str());

                    let mut file = File::create(&file_path[..]).unwrap();
                    let events_ref_clone = events_ref_clone.lock().unwrap();
                    let encoded : Vec<u8> = bincode::serialize(&*events_ref_clone).unwrap();
                    let _result = file.write_all(&encoded);

                    // set label text to file name
                    if file_name.len() > 12 {
                        script_file_label_clone.set_text(format!("{}...", &file_name[0..12]).as_str());
                    } else {
                        script_file_label_clone.set_text(file_name.as_str());
                    }
                    dialog.emit_close();
                }

                halt_actions_ref_clone.store(false, Ordering::Relaxed);
            });

            dialog.run();
        });

        let infinite_loop_ref = Arc::clone(&infinite_loop);
        let infinite_loop_checkbox : gtk::CheckButton = builder.object("checkbox_infinite").expect("Couldn't get gtk object 'checkbox_infinite'");
        infinite_loop_checkbox.connect_toggled(move |button| {
            infinite_loop_ref.store(button.is_active(), Ordering::Relaxed);
        });

        let delay_ref = Arc::clone(&delay);
        let delay_checkbox : gtk::CheckButton = builder.object("checkbox_delay").expect("Couldn't get gtk object 'checkbox_delay'");
        delay_checkbox.connect_toggled(move |button| {
            delay_ref.store(button.is_active(), Ordering::Relaxed);
        });

        let loop_count_ref = Arc::clone(&loop_count);
        let loop_count_button : gtk::SpinButton = builder.object("loop_count").expect("Couldn't get gtk object 'loop_count'");
        loop_count_button.connect_value_notify(move |button| {
            let loop_count_button_val = button.value_as_int();
            let mut loop_count_val = loop_count_ref.lock().unwrap();
            *loop_count_val = loop_count_button_val;
        });

        let window_ref : gtk::Window = builder.object("window").expect("Couldn't get gtk object 'window'");
        let script_file_label: gtk::Label = builder.object("label_script_file").expect("Couldn't get gtk object 'label_script_file'.");
        let events_ref = Arc::clone(&events);
        record_button.connect_clicked(move |_| {
            if minimize.is_active() {
                window_ref.iconify();
            }

            if !record_ref.load(Ordering::Relaxed) {
                script_file_label.set_text("new");
                log("Recording...");
                record_ref.store(true, Ordering::Relaxed);
                events_ref.lock().unwrap().clear();
            }
        });

        let stop_recording_button : gtk::Button = builder.object("button_stop_recording").expect("Couldn't get gtk object 'button_stop_recording'");
        let record_ref = Arc::clone(&record);
        stop_recording_button.connect_clicked(move |_| {
            if record_ref.load(Ordering::Relaxed) {
                log("Stopped recording...");
                record_ref.store(false, Ordering::Relaxed);
            }
        });

        let run_button: gtk::Button = builder.object("button_run").expect("Couldn't get gtk object 'button_run'");
        let run_ref = Arc::clone(&run);
        let window_ref : gtk::Window = builder.object("window").expect("Couldn't get gtk object 'window'");
        let minimize : gtk::CheckButton = builder.object("checkbox_minimize").expect("Couldn't get gtk object 'checkbox_minimize'");
        run_button.connect_clicked(move |_| {
            if minimize.is_active() {
                window_ref.iconify();
            }

            if !run_ref.load(Ordering::Relaxed) {
                log("Running...");
                run_ref.store(true, Ordering::Relaxed);
            } else if run_ref.load(Ordering::Relaxed) {
                log("Stopped running...");
                run_ref.store(false, Ordering::Relaxed);
            }
        });

        window.set_application(Some(app));
        window.show_all();
    });
    app.run();
}

fn event_loop(events: Arc<Mutex<Vec<Event>>>, run: Arc<AtomicBool>, infinite_loop: Arc<AtomicBool>, loop_count: Arc<Mutex<i32>>, delay: Arc<AtomicBool>) {
    loop {
        if run.load(Ordering::Relaxed) {
            let events_ref = Arc::clone(&events);
            let run_ref = Arc::clone(&run);
            let infinite_loop_ref = Arc::clone(&infinite_loop);
            let loop_count_ref = Arc::clone(&loop_count);
            let delay_ref = Arc::clone(&delay);
            send_events(events_ref, run_ref, infinite_loop_ref, loop_count_ref, delay_ref);
        }
    }
}

fn send_events(events: Arc<Mutex<Vec<Event>>>, run: Arc<AtomicBool>, infinite_loop: Arc<AtomicBool>, loop_count: Arc<Mutex<i32>>, delay: Arc<AtomicBool>) {
    let events = events.lock().unwrap().to_vec();
    if events.is_empty() {
        log("There aren't any events to run!");
        run.store(false, Ordering::Relaxed);
        return;
    }

    let loop_count = loop_count.lock().unwrap();
    let mut i = 0;
    while i < *loop_count {
        let mut last_time = events[0].time;
        for event in &events {
            // Running can be disabled while in the middle of running so we have to check if flag is still true
            if run.load(Ordering::Relaxed) {
                let mut wait_duration = event.time.duration_since(last_time).unwrap();
                last_time = event.time;
                if delay.load(Ordering::Relaxed) {
                    // Shorten all durations by a small percentage to account for delay in other misc things
                    wait_duration = wait_duration.mul_f64(0.98);
                } else {
                    wait_duration = Duration::from_micros(50);
                }
                spin_sleep::sleep(wait_duration);
                send_event(&event.event_type);
            } else {
                log("Running halted!");
                infinite_loop.store(false, Ordering::Relaxed);
                break;
            }
        }

        // Send release key event for stop recording button
        match simulate(&EventType::KeyRelease(Key::Dot)) {
            Ok(()) => (),
            Err(SimulateError) => {
                eprintln!("Could not send final release key.");
            }
        };
        
        if !infinite_loop.load(Ordering::Relaxed) {
            i += 1;
        }
    }
    
    run.store(false, Ordering::Relaxed);

    log("Done");
}

