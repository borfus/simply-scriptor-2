use rdev::{listen, Event, EventType, simulate, SimulateError, Key};
use std::{thread, sync::mpsc::Sender, sync::mpsc::Receiver, sync::Arc, sync::Mutex, time::SystemTime};

extern crate chrono;
use chrono::offset::Utc;
use chrono::DateTime;

pub fn spawn_event_listener(sendch: Sender<Event>) {
    let _listener = thread::spawn(move || {
        listen(move |event| {
            sendch.send(event)
                  .unwrap_or_else(|e| log(format!("Could not send event {:?}", e).as_str()));
        })
        .expect("Could not listen");
    });
}

pub fn spawn_event_receiver(recvch: Receiver<Event>, record: Arc<Mutex<bool>>, run: Arc<Mutex<bool>>, events: Arc<Mutex<Vec<Event>>>, halt_actions: Arc<Mutex<bool>>) {
    thread::spawn(move || {
        for event in recvch.iter() {
            let halt_actions = halt_actions.lock().unwrap();
            if *halt_actions {
                continue;
            }

            let mut record = record.lock().unwrap();
            let mut run = run.lock().unwrap();
            if event.event_type == EventType::KeyRelease(Key::Comma) {
                if !*record {
                    *record = true;
                    log("Recording...");
                    events.lock().unwrap().clear();
                    continue;
                }
            }

            if event.event_type == EventType::KeyRelease(Key::Dot) {
                if *record {
                    *record = false;
                    log("Stopped recording...");
                    continue;
                }
            }

            if event.event_type == EventType::KeyRelease(Key::Slash) {
                if !*run && !*record {
                    log("Running...");
                    *run = true;
                } else if *run {
                    log("Stopped running...");
                    *run = false;
                }
                continue;
            }

            if *record && !*run {
                events.lock().unwrap().push(event);
            }
        }
    });
}

pub fn send_event(event_type: &EventType) {
    match simulate(event_type) {
        Ok(()) => (),
        Err(SimulateError) => {
            eprintln!("Could not send event: {:?}", event_type);
        }
    }
}

fn get_time() -> String {
    let system_time = SystemTime::now();
    let datetime: DateTime<Utc> = system_time.into();
    format!("{}", datetime.format("%d/%m/%Y %T"))
}


pub fn log(message: &str) {
    println!("{}: {}", get_time(), message);
}

