use rdev::{listen, simulate, Button, Event, EventType, Key, SimulateError};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::SystemTime,
};

extern crate chrono;
use chrono::{offset::Utc, DateTime};

// Spawn new thread to listen for any keyboard or mouse input
// Sends events through a tunnel that must be set up before calling this function
pub fn spawn_event_listener(sendch: Sender<Event>) {
    let _listener = thread::spawn(move || {
        listen(move |event| {
            sendch
                .send(event)
                .unwrap_or_else(|e| log(format!("Could not send event {:?}", e).as_str()));
        })
        .expect("Could not listen");
    });
}

// Listen for events from a tunnel sender and set appropriate flags for main program
// Used to handle keyboard shortcuts for recording, stop recording, and running scripts
pub fn spawn_event_receiver(
    recvch: Receiver<Event>,
    record: Arc<AtomicBool>,
    run: Arc<AtomicBool>,
    events: Arc<Mutex<Vec<Event>>>,
    halt_actions: Arc<AtomicBool>,
) {
    thread::spawn(move || {
        for event in recvch.iter() {
            // Debug: Log all events on macOS to see what's being captured
            #[cfg(target_os = "macos")]
            {
                match &event.event_type {
                    EventType::ButtonPress(btn) => log(&format!("Captured ButtonPress: {:?}", btn)),
                    EventType::ButtonRelease(btn) => {
                        log(&format!("Captured ButtonRelease: {:?}", btn))
                    }
                    _ => {}
                }
            }

            if halt_actions.load(Ordering::Relaxed) {
                continue;
            }

            if event.event_type == EventType::KeyRelease(Key::Comma)
                && !record.load(Ordering::Relaxed)
            {
                record.store(true, Ordering::Relaxed);
                log("Recording...");
                events.lock().unwrap().clear();
                continue;
            }

            if event.event_type == EventType::KeyRelease(Key::Dot) && record.load(Ordering::Relaxed)
            {
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

// Simulate the previously recorded input keyboard/mouse input event
pub fn send_event(event_type: &EventType) {
    // On macOS, mouse button events sometimes need special handling
    #[cfg(target_os = "macos")]
    {
        match event_type {
            EventType::ButtonPress(button) | EventType::ButtonRelease(button) => {
                // For macOS, we need to make sure button events are sent properly
                // Try simulating the event multiple times if it fails
                for attempt in 0..3 {
                    match simulate(event_type) {
                        Ok(()) => return,
                        Err(SimulateError) => {
                            if attempt == 2 {
                                eprintln!(
                                    "Could not send button event after 3 attempts: {:?}",
                                    event_type
                                );
                            } else {
                                // Small delay before retry
                                std::thread::sleep(std::time::Duration::from_micros(100));
                            }
                        }
                    }
                }
            }
            _ => match simulate(event_type) {
                Ok(()) => (),
                Err(SimulateError) => {
                    eprintln!("Could not send event: {:?}", event_type);
                }
            },
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        match simulate(event_type) {
            Ok(()) => (),
            Err(SimulateError) => {
                eprintln!("Could not send event: {:?}", event_type);
            }
        }
    }
}

pub fn log(message: &str) {
    println!("{}: {}", get_time(), message);
}

fn get_time() -> String {
    let system_time = SystemTime::now();
    let datetime: DateTime<Utc> = system_time.into();
    format!("{}", datetime.format("%d/%m/%Y %T"))
}

