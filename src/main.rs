use rdev::{listen, simulate, Event, EventType, SimulateError, Key};
use std::{thread, time, time::SystemTime, sync::Arc, sync::Mutex, sync::mpsc::channel};

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
    thread::spawn(move || {
        for event in recvch.iter() {
            // println!("Received {:?}", event.event_type);
            let mut record_ref = record_ref.lock().unwrap();
            if event.event_type == EventType::KeyRelease(Key::F10) {
                *record_ref = !*record_ref;
                if *record_ref {
                    println!("Recording...");
                } else {
                    println!("Stopped recording...");
                }
            }

            if *record_ref {
                events_ref.lock().unwrap().push(event);
            }
        }
    });

    loop {
        let record = record.lock().unwrap();
        let mut run = run.lock().unwrap();
        let mut events = events.lock().unwrap();

        if *run && !*record {
            send_events(events.to_vec());
            *run = false;
        }

        for event in events.to_vec() {
            if *record {
                println!("{:?}", event.event_type);
            }
        }
        events.clear();
    };
}

fn send_events(events: Vec<Event>) {
    let wait_duration = SystemTime::now();
    for event in events {
        send_event(&event.event_type);
        let wait_duration = event.time.duration_since(wait_duration);
        thread::sleep(wait_duration.unwrap());
    }
}

fn send_event(event_type: &EventType) {
    let delay = time::Duration::from_millis(20);
    match simulate(event_type) {
        Ok(()) => (),
        Err(SimulateError) => {
            eprintln!("Could not send event: {:?}", event_type);
        }
    }

    thread::sleep(delay);
}

