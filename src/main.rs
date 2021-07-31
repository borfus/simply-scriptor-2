use rdev::{simulate, Button, EventType, Key, SimulateError};
use std::{thread, time};

fn main() {
    send(&EventType::MouseMove { x: 200.0, y: 200.0 });
    send(&EventType::ButtonPress(Button::Left));
    send(&EventType::ButtonRelease(Button::Left));
    send(&EventType::KeyPress(Key::KeyS));
    send(&EventType::KeyRelease(Key::KeyS));
}

fn send(event_type: &EventType) {
    let delay = time::Duration::from_millis(20);
    match simulate(event_type) {
        Ok(()) => (),
        Err(SimulateError) => {
            eprintln!("Could not send event: {:?}", event_type);
        }
    }

    thread::sleep(delay);
}

