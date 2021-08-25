use rdev::{simulate, SimulateError, Event, EventType, Key};
use std::{thread, sync::Arc, sync::Mutex, sync::mpsc::channel, time::Duration};
use simply_scriptor_2::*;
use gtk::prelude::*;

fn main() {
    let (sendch, recvch) = channel();
    spawn_event_listener(sendch);

    let record = Arc::new(Mutex::new(false));
    let run = Arc::new(Mutex::new(false));
    let events = Arc::new(Mutex::new(Vec::new()));
    let infinite_loop = Arc::new(Mutex::new(false));
    let loop_count = Arc::new(Mutex::new(1));
    let delay = Arc::new(Mutex::new(false));

    let record_ref = Arc::clone(&record);
    let run_ref = Arc::clone(&run);
    let events_ref = Arc::clone(&events);
    spawn_event_receiver(recvch, record_ref, run_ref, events_ref);

    let run_ref = Arc::clone(&run);
    let events_ref = Arc::clone(&events);
    let infinite_loop_ref = Arc::clone(&infinite_loop);
    let loop_count_ref = Arc::clone(&loop_count);
    let delay_ref = Arc::clone(&delay);
    thread::spawn(move || {
        loop {
            let run_val = run_ref.lock().unwrap();

            if *run_val {
                drop(run_val);
                let events_ref_clone = Arc::clone(&events_ref);
                let run_ref_clone = Arc::clone(&run_ref);
                let infinite_loop_clone = Arc::clone(&infinite_loop_ref);
                let loop_count_clone = Arc::clone(&loop_count_ref);
                let delay_ref_clone = Arc::clone(&delay_ref);
                send_events(events_ref_clone, run_ref_clone, infinite_loop_clone, loop_count_clone, delay_ref_clone);
            }
        }
    });

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
        let window_ref : gtk::Window = builder.object("window").expect("Couldn't get gtk object 'window'");
        let minimize : gtk::CheckButton = builder.object("checkbox_minimize").expect("Couldn't get gtk object 'checkbox_minimize'");
        record_button.connect_clicked(move |_| {
            if minimize.is_active() {
                window_ref.iconify();
            }

            let mut record_val = record_ref.lock().unwrap();
            if !*record_val {
                log("Recording...");
                *record_val = true;
                events_ref.lock().unwrap().clear();
            }
        });

        let stop_recording_button : gtk::Button = builder.object("button_stop_recording").expect("Couldn't get gtk object 'button_stop_recording'");
        let record_ref = Arc::clone(&record);
        stop_recording_button.connect_clicked(move |_| {
            let mut record_val = record_ref.lock().unwrap();
            if *record_val {
                log("Stopped recording...");
                *record_val = false;
            }
        });

        let run_button: gtk::Button = builder.object("button_run").expect("Couldn't get gtk object 'button_run'");
        let run_ref = Arc::clone(&run);
        let loop_count_ref = Arc::clone(&loop_count);
        let infinite_loop_ref = Arc::clone(&infinite_loop);
        let delay_ref = Arc::clone(&delay);
        let window_ref : gtk::Window = builder.object("window").expect("Couldn't get gtk object 'window'");
        let minimize : gtk::CheckButton = builder.object("checkbox_minimize").expect("Couldn't get gtk object 'checkbox_minimize'");
        run_button.connect_clicked(move |_| {
            if minimize.is_active() {
                window_ref.iconify();
            }

            let mut run_val = run_ref.lock().unwrap();

            if !*run_val {
                let infinite_loop_checkbox : gtk::CheckButton = builder.object("checkbox_infinite").expect("Couldn't get gtk object 'checkbox_infinite'");
                let infinite_loop_checkbox_val = infinite_loop_checkbox.is_active();
                let mut infinite_loop_val = infinite_loop_ref.lock().unwrap();
                *infinite_loop_val = infinite_loop_checkbox_val;

                let delay_checkbox : gtk::CheckButton = builder.object("checkbox_delay").expect("Couldn't get gtk object 'checkbox_delay'");
                let delay_checkbox_val = delay_checkbox.is_active();
                let mut delay_val = delay_ref.lock().unwrap();
                *delay_val = delay_checkbox_val;

                let loop_count_button : gtk::SpinButton = builder.object("loop_count").expect("Couldn't get gtk object 'loop_count'");
                let loop_count_button_val = loop_count_button.value_as_int();
                let mut loop_count_val = loop_count_ref.lock().unwrap();
                *loop_count_val = loop_count_button_val;

                log("Running...");
                *run_val = true;
            } else if *run_val {
                log("Stopped running...");
                *run_val = false;
            }
        });

        window.set_application(Some(app));
        window.show_all();
    });
    app.run();
}

fn send_events(events: Arc<Mutex<Vec<Event>>>, run: Arc<Mutex<bool>>, infinite_loop: Arc<Mutex<bool>>, loop_count: Arc<Mutex<i32>>, delay: Arc<Mutex<bool>>) {
    let events = events.lock().unwrap().to_vec();
    if events.len() == 0 {
        log("There aren't any events to run!");
        let mut run = run.lock().unwrap();
        *run = false;
        return;
    }

    let delay = delay.lock().unwrap();

    let mut infinite_loop = infinite_loop.lock().unwrap();
    let loop_count = loop_count.lock().unwrap();
    let mut i = 0;
    while i < *loop_count {
        let mut last_time = events[0].time;
        for event in &events {
            // Running can be disabled while in the middle of running so we have to check if flag is still true
            let run_ref = Arc::clone(&run);
            let run_ref = run_ref.lock().unwrap();
            if *run_ref {
                send_event(&event.event_type);
                let wait_duration = event.time.duration_since(last_time).unwrap();
                last_time = event.time;
                drop(run_ref);
                if *delay {
                    spin_sleep::sleep(wait_duration);
                } else {
                    spin_sleep::sleep(Duration::from_micros(50));
                }
            } else {
                log("Running halted!");
                *infinite_loop = false;
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
        
        if !*infinite_loop {
            i += 1;
        }
    }
    
    let mut run = run.lock().unwrap();
    *run = false;

    log("Done");
}

