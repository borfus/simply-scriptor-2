use rdev::{listen, Event, EventType, simulate, SimulateError, Key};
use std::{thread, sync::{mpsc::{Sender, Receiver}, Arc, Mutex, atomic::{AtomicBool, Ordering}}, time::SystemTime};

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

pub fn spawn_event_receiver(
    recvch: Receiver<Event>,
    record: Arc<AtomicBool>,
    run: Arc<AtomicBool>,
    events: Arc<Mutex<Vec<Event>>>,
    halt_actions: Arc<AtomicBool>
) {
    thread::spawn(move || {
        for event in recvch.iter() {
            if halt_actions.load(Ordering::Relaxed) {
                continue;
            }

            if event.event_type == EventType::KeyRelease(Key::Comma) && !record.load(Ordering::Relaxed) {
                record.store(true, Ordering::Relaxed);
                log("Recording...");
                events.lock().unwrap().clear();
                continue;
            }

            if event.event_type == EventType::KeyRelease(Key::Dot) && record.load(Ordering::Relaxed) {
                record.store(false, Ordering::Relaxed);
                log("Stopped recording...");
                continue;
            }

            if event.event_type == EventType::KeyRelease(Key::Slash) {
                if !run.load(Ordering::Relaxed) && !record.load(Ordering::Relaxed) {
                    log("Running...");
                    run.store(true, Ordering::Relaxed);
                } else if run.load(Ordering::Relaxed) {
                    log("Stopped running...");
                    run.store(false, Ordering::Relaxed);
                }
                continue;
            }

            if record.load(Ordering::Relaxed) && !run.load(Ordering::Relaxed) {
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

