use rdev::{simulate, SimulateError, Event, EventType, Key};
use std::{thread, sync::Arc, sync::Mutex, sync::mpsc::channel};
use simply_scriptor::*;

fn main() {
    let (sendch, recvch) = channel();
    spawn_event_listener(sendch);

    let record = Arc::new(Mutex::new(false));
    let run = Arc::new(Mutex::new(false));
    let events = Arc::new(Mutex::new(Vec::new()));

    let record_ref = Arc::clone(&record);
    let run_ref = Arc::clone(&run);
    let events_ref = Arc::clone(&events);
    spawn_event_receiver(recvch, record_ref, run_ref, events_ref);

    loop {
        let record_val = record.lock().unwrap();
        let run_val = run.lock().unwrap();

        if *run_val && !*record_val {
            drop(run_val);
            let events_ref = Arc::clone(&events);
            let run_ref = Arc::clone(&run);
            send_events(events_ref, run_ref);
        }
    };
}

fn send_events(events: Arc<Mutex<Vec<Event>>>, run: Arc<Mutex<bool>>) {
    let events = events.lock().unwrap().to_vec();
    let mut run = run.lock().unwrap();
    if events.len() == 0 {
        println!("There aren't any events to run!");
        *run = false;
        return;
    }

    let mut last_time = events[0].time;
    for event in events {
        // Running can be disabled while in the middle of running so we have to check if flag is still true
        if *run {
            send_event(&event.event_type);
            let wait_duration = event.time.duration_since(last_time).unwrap();
            last_time = event.time;
            println!("{:?}", wait_duration);
            thread::sleep(wait_duration);
        } else {
            println!("Running halted!");
            break;
        }
    }
    
    // Send release key event for stop recording button
    match simulate(&EventType::KeyRelease(Key::F10)) {
        Ok(()) => (),
        Err(SimulateError) => {
            eprintln!("Could not send final release key.");
        }
    };
    *run = false;
}

