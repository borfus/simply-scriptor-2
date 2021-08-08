use rdev::{listen, simulate, Event, EventType, SimulateError, Key};
use std::{thread, sync::Arc, sync::Mutex, sync::mpsc::channel};

fn main() {
    // spawn new thread because listen blocks
    let (sendch, recvch) = channel();
    let _listener = thread::spawn(move || {
        listen(move |event| {
            sendch.send(event)
                .unwrap_or_else(|e| println!("Could not send event {:?}", e));
        })
        .expect("Could not listen");
    });

    let record = Arc::new(Mutex::new(false));
    let run = Arc::new(Mutex::new(false));
    let events = Arc::new(Mutex::new(Vec::new()));

    let events_ref = Arc::clone(&events);
    let record_ref = Arc::clone(&record);
    let run_ref = Arc::clone(&run);
    thread::spawn(move || {
        for event in recvch.iter() {
            let mut record_ref = record_ref.lock().unwrap();
            let mut run_ref = run_ref.lock().unwrap();
            if event.event_type == EventType::KeyRelease(Key::F10) {
                *record_ref = !*record_ref;
                if *record_ref {
                    println!("Recording...");
                    events_ref.lock().unwrap().clear();
                } else {
                    println!("Stopped recording...");
                }
            }

            if event.event_type == EventType::KeyRelease(Key::F12) {
                *run_ref = !*run_ref;
                if *run_ref && !*record_ref {
                    println!("Running...");
                } else if !*run_ref {
                    println!("Stopped running...");
                }
            }

            if *record_ref {
                events_ref.lock().unwrap().push(event);
            }
        }
    });

    loop {
        let record = record.lock().unwrap();
        let run_val = run.lock().unwrap();

        if *run_val && !*record {
            let events_ref = Arc::clone(&events);
            let run_ref = Arc::clone(&run);
            thread::spawn(move || {
                send_events(events_ref, run_ref);
            });
        }
    };
}

fn send_events(events: Arc<Mutex<Vec<Event>>>, run: Arc<Mutex<bool>>) {
    let events = events.lock().unwrap().to_vec();
    let mut last_time = events[0].time;
    let mut run = run.lock().unwrap();
    for event in events {
        // Running can be disabled while in the middle of running so we have to check if flag is still true
        if *run {
            send_event(&event.event_type);
            let wait_duration = event.time.duration_since(last_time).unwrap();
            last_time = event.time;
            thread::sleep(wait_duration);
        } else {
            println!("Running halted!");
            break;
        }
    }
    *run = false;
}

fn send_event(event_type: &EventType) {
    match simulate(event_type) {
        Ok(()) => (),
        Err(SimulateError) => {
            eprintln!("Could not send event: {:?}", event_type);
        }
    }
}

