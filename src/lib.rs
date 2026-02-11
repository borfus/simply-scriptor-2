#[cfg(not(target_os = "macos"))]
use rdev::{listen, simulate, SimulateError};
use rdev::{Event, EventType, Key};
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

#[cfg(target_os = "macos")]
mod macos_events;

// Spawn new thread to listen for any keyboard or mouse input
// Sends events through a tunnel that must be set up before calling this function
pub fn spawn_event_listener(sendch: Sender<Event>) {
    #[cfg(target_os = "macos")]
    {
        // On macOS, use CGEventTap for reliable event capture
        macos_events::start_macos_event_tap(sendch);
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _listener = thread::spawn(move || {
            listen(move |event| {
                sendch
                    .send(event)
                    .unwrap_or_else(|e| log(format!("Could not send event {:?}", e).as_str()));
            })
            .expect("Could not listen");
        });
    }
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
    #[cfg(target_os = "macos")]
    {
        // On macOS, use CGEvent for simulation
        if let Err(e) = macos_events::simulate_macos_event(event_type) {
            eprintln!("Could not send event on macOS: {} - {:?}", e, event_type);
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

