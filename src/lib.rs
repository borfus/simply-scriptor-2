use rdev::{listen, Event, EventType, simulate, SimulateError, Key};
use std::{thread, sync::mpsc::Sender, sync::mpsc::Receiver, sync::Arc, sync::Mutex};

pub fn spawn_event_listener(sendch: Sender<Event>) {
    let _listener = thread::spawn(move || {
        listen(move |event| {
            sendch.send(event)
                  .unwrap_or_else(|e| println!("Could not send event {:?}", e));
        })
        .expect("Could not listen");
    });
}

pub fn spawn_event_receiver(recvch: Receiver<Event>, record: Arc<Mutex<bool>>, run: Arc<Mutex<bool>>, events: Arc<Mutex<Vec<Event>>>) {
    thread::spawn(move || {
        for event in recvch.iter() {
            let mut record = record.lock().unwrap();
            let mut run = run.lock().unwrap();
            if event.event_type == EventType::KeyRelease(Key::F9) {
                if !*record {
                    *record = true;
                    println!("Recording...");
                    events.lock().unwrap().clear();
                    continue;
                }
            }

            if event.event_type == EventType::KeyRelease(Key::F10) {
                if *record {
                    *record = false;
                    println!("Stopped recording...");
                    continue;
                }
            }

            if event.event_type == EventType::KeyRelease(Key::F12) {
                println!("F12 detected!");
                if !*run && !*record {
                    println!("Running...");
                    *run = true;
                } else if *run {
                    println!("Stopped running...");
                    *run = false;
                    continue;
                }
            }

            if *record && !*run {
                events.lock().unwrap().push(event);
            }
        }
    });
}

// fn determine_action_state() {
// }

pub fn send_event(event_type: &EventType) {
    match simulate(event_type) {
        Ok(()) => (),
        Err(SimulateError) => {
            eprintln!("Could not send event: {:?}", event_type);
        }
    }
}

