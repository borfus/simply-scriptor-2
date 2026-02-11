#[cfg(target_os = "macos")]
use core_graphics::event::{CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType, EventField};
#[cfg(target_os = "macos")]
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
#[cfg(target_os = "macos")]
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
#[cfg(target_os = "macos")]
use std::sync::mpsc::Sender;
#[cfg(target_os = "macos")]
use rdev::{Event, EventType, Key, Button};
#[cfg(target_os = "macos")]
use std::time::SystemTime;

#[cfg(target_os = "macos")]
pub fn start_macos_event_tap(sender: Sender<Event>) {
    use std::thread;
    
    thread::spawn(move || {
        // Create event tap for all events
        let event_mask = CGEventType::LeftMouseDown as u64
            | CGEventType::LeftMouseUp as u64
            | CGEventType::RightMouseDown as u64
            | CGEventType::RightMouseUp as u64
            | CGEventType::MouseMoved as u64
            | CGEventType::LeftMouseDragged as u64
            | CGEventType::RightMouseDragged as u64
            | CGEventType::KeyDown as u64
            | CGEventType::KeyUp as u64
            | CGEventType::ScrollWheel as u64
            | CGEventType::OtherMouseDown as u64
            | CGEventType::OtherMouseUp as u64;

        let sender_clone = sender.clone();
        
        match CGEventTap::new(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly,
            event_mask,
            move |_proxy, event_type, cg_event| {
                if let Some(event) = convert_cg_event_to_rdev(event_type, &cg_event) {
                    let _ = sender_clone.send(event);
                }
                // Pass through the event unchanged
                Some(cg_event)
            },
        ) {
            Ok(tap) => {
                unsafe {
                    let loop_source = tap.mach_port.create_runloop_source(0).unwrap();
                    let current_loop = CFRunLoop::get_current();
                    current_loop.add_source(&loop_source, kCFRunLoopCommonModes);
                    tap.enable();
                    CFRunLoop::run_current();
                }
            }
            Err(e) => {
                eprintln!("Failed to create CGEventTap: {:?}", e);
                eprintln!("Make sure the app has Accessibility permissions in System Settings!");
            }
        }
    });
}

#[cfg(target_os = "macos")]
fn convert_cg_event_to_rdev(event_type: CGEventType, cg_event: &CGEvent) -> Option<Event> {
    let time = SystemTime::now();
    
    let event_type = match event_type {
        CGEventType::LeftMouseDown => {
            EventType::ButtonPress(Button::Left)
        }
        CGEventType::LeftMouseUp => {
            EventType::ButtonRelease(Button::Left)
        }
        CGEventType::RightMouseDown => {
            EventType::ButtonPress(Button::Right)
        }
        CGEventType::RightMouseUp => {
            EventType::ButtonRelease(Button::Right)
        }
        CGEventType::OtherMouseDown => {
            EventType::ButtonPress(Button::Middle)
        }
        CGEventType::OtherMouseUp => {
            EventType::ButtonRelease(Button::Middle)
        }
        CGEventType::MouseMoved | CGEventType::LeftMouseDragged | CGEventType::RightMouseDragged => {
            let location = cg_event.location();
            EventType::MouseMove {
                x: location.x,
                y: location.y,
            }
        }
        CGEventType::ScrollWheel => {
            let delta_y = cg_event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_1);
            let delta_x = cg_event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_2);
            EventType::Wheel {
                delta_x,
                delta_y,
            }
        }
        CGEventType::KeyDown => {
            let keycode = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
            EventType::KeyPress(macos_keycode_to_rdev_key(keycode as u16))
        }
        CGEventType::KeyUp => {
            let keycode = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
            EventType::KeyRelease(macos_keycode_to_rdev_key(keycode as u16))
        }
        _ => return None,
    };

    Some(Event {
        time,
        event_type,
        unicode: None,
        platform_code: 0,
        position_code: 0,
        extra_data: 0,
        usb_hid: 0,
    })
}

#[cfg(target_os = "macos")]
fn macos_keycode_to_rdev_key(keycode: u16) -> Key {
    // macOS virtual key codes mapped to rdev keys
    match keycode {
        0x00 => Key::KeyA,
        0x01 => Key::KeyS,
        0x02 => Key::KeyD,
        0x03 => Key::KeyF,
        0x04 => Key::KeyH,
        0x05 => Key::KeyG,
        0x06 => Key::KeyZ,
        0x07 => Key::KeyX,
        0x08 => Key::KeyC,
        0x09 => Key::KeyV,
        0x0B => Key::KeyB,
        0x0C => Key::KeyQ,
        0x0D => Key::KeyW,
        0x0E => Key::KeyE,
        0x0F => Key::KeyR,
        0x10 => Key::KeyY,
        0x11 => Key::KeyT,
        0x12 => Key::Num1,
        0x13 => Key::Num2,
        0x14 => Key::Num3,
        0x15 => Key::Num4,
        0x16 => Key::Num6,
        0x17 => Key::Num5,
        0x18 => Key::Equal,
        0x19 => Key::Num9,
        0x1A => Key::Num7,
        0x1B => Key::Minus,
        0x1C => Key::Num8,
        0x1D => Key::Num0,
        0x1E => Key::RightBracket,
        0x1F => Key::KeyO,
        0x20 => Key::KeyU,
        0x21 => Key::LeftBracket,
        0x22 => Key::KeyI,
        0x23 => Key::KeyP,
        0x24 => Key::Return,
        0x25 => Key::KeyL,
        0x26 => Key::KeyJ,
        0x27 => Key::Quote,
        0x28 => Key::KeyK,
        0x29 => Key::SemiColon,
        0x2A => Key::BackSlash,
        0x2B => Key::Comma,
        0x2C => Key::Slash,
        0x2D => Key::KeyN,
        0x2E => Key::KeyM,
        0x2F => Key::Dot,
        0x30 => Key::Tab,
        0x31 => Key::Space,
        0x32 => Key::BackQuote,
        0x33 => Key::Backspace,
        0x35 => Key::Escape,
        0x37 => Key::MetaLeft,
        0x38 => Key::ShiftLeft,
        0x39 => Key::CapsLock,
        0x3A => Key::Alt,
        0x3B => Key::ControlLeft,
        0x3C => Key::ShiftRight,
        0x3D => Key::AltGr,
        0x3E => Key::ControlRight,
        0x3F => Key::Function,
        0x7A => Key::F1,
        0x78 => Key::F2,
        0x63 => Key::F3,
        0x76 => Key::F4,
        0x60 => Key::F5,
        0x61 => Key::F6,
        0x62 => Key::F7,
        0x64 => Key::F8,
        0x65 => Key::F9,
        0x6D => Key::F10,
        0x67 => Key::F11,
        0x6F => Key::F12,
        0x72 => Key::Insert,
        0x73 => Key::Home,
        0x74 => Key::PageUp,
        0x75 => Key::Delete,
        0x77 => Key::End,
        0x79 => Key::PageDown,
        0x7B => Key::LeftArrow,
        0x7C => Key::RightArrow,
        0x7D => Key::DownArrow,
        0x7E => Key::UpArrow,
        _ => Key::Unknown(keycode as u32),
    }
}

#[cfg(target_os = "macos")]
pub fn simulate_macos_event(event_type: &EventType) -> Result<(), String> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| "Failed to create event source")?;

    match event_type {
        EventType::KeyPress(key) => {
            if let Some(keycode) = rdev_key_to_macos_keycode(key) {
                let event = CGEvent::new_keyboard_event(source, keycode, true)
                    .map_err(|_| "Failed to create key press event")?;
                event.post(CGEventTapLocation::HID);
            }
        }
        EventType::KeyRelease(key) => {
            if let Some(keycode) = rdev_key_to_macos_keycode(key) {
                let event = CGEvent::new_keyboard_event(source, keycode, false)
                    .map_err(|_| "Failed to create key release event")?;
                event.post(CGEventTapLocation::HID);
            }
        }
        EventType::ButtonPress(button) => {
            let (event_type, mouse_button) = match button {
                Button::Left => (CGEventType::LeftMouseDown, core_graphics::event::CGMouseButton::Left),
                Button::Right => (CGEventType::RightMouseDown, core_graphics::event::CGMouseButton::Right),
                Button::Middle => (CGEventType::OtherMouseDown, core_graphics::event::CGMouseButton::Center),
                _ => return Err("Unknown button".to_string()),
            };
            
            if let Some(location) = CGEvent::new(source.clone()).ok().and_then(|e| Some(e.location())) {
                let event = CGEvent::new_mouse_event(source, event_type, location, mouse_button)
                    .map_err(|_| "Failed to create mouse button press event")?;
                event.post(CGEventTapLocation::HID);
            }
        }
        EventType::ButtonRelease(button) => {
            let (event_type, mouse_button) = match button {
                Button::Left => (CGEventType::LeftMouseUp, core_graphics::event::CGMouseButton::Left),
                Button::Right => (CGEventType::RightMouseUp, core_graphics::event::CGMouseButton::Right),
                Button::Middle => (CGEventType::OtherMouseUp, core_graphics::event::CGMouseButton::Center),
                _ => return Err("Unknown button".to_string()),
            };
            
            if let Some(location) = CGEvent::new(source.clone()).ok().and_then(|e| Some(e.location())) {
                let event = CGEvent::new_mouse_event(source, event_type, location, mouse_button)
                    .map_err(|_| "Failed to create mouse button release event")?;
                event.post(CGEventTapLocation::HID);
            }
        }
        EventType::MouseMove { x, y } => {
            let point = core_graphics::geometry::CGPoint::new(*x, *y);
            let event = CGEvent::new_mouse_event(
                source,
                CGEventType::MouseMoved,
                point,
                core_graphics::event::CGMouseButton::Left,
            )
            .map_err(|_| "Failed to create mouse move event")?;
            event.post(CGEventTapLocation::HID);
        }
        EventType::Wheel { delta_x, delta_y } => {
            let event = CGEvent::new_scroll_event(
                source,
                core_graphics::event::ScrollEventUnit::PIXEL,
                2,
                *delta_y as i32,
                *delta_x as i32,
                0,
            )
            .map_err(|_| "Failed to create scroll event")?;
            event.post(CGEventTapLocation::HID);
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn rdev_key_to_macos_keycode(key: &Key) -> Option<u16> {
    let keycode = match key {
        Key::KeyA => 0x00,
        Key::KeyS => 0x01,
        Key::KeyD => 0x02,
        Key::KeyF => 0x03,
        Key::KeyH => 0x04,
        Key::KeyG => 0x05,
        Key::KeyZ => 0x06,
        Key::KeyX => 0x07,
        Key::KeyC => 0x08,
        Key::KeyV => 0x09,
        Key::KeyB => 0x0B,
        Key::KeyQ => 0x0C,
        Key::KeyW => 0x0D,
        Key::KeyE => 0x0E,
        Key::KeyR => 0x0F,
        Key::KeyY => 0x10,
        Key::KeyT => 0x11,
        Key::Num1 => 0x12,
        Key::Num2 => 0x13,
        Key::Num3 => 0x14,
        Key::Num4 => 0x15,
        Key::Num6 => 0x16,
        Key::Num5 => 0x17,
        Key::Equal => 0x18,
        Key::Num9 => 0x19,
        Key::Num7 => 0x1A,
        Key::Minus => 0x1B,
        Key::Num8 => 0x1C,
        Key::Num0 => 0x1D,
        Key::RightBracket => 0x1E,
        Key::KeyO => 0x1F,
        Key::KeyU => 0x20,
        Key::LeftBracket => 0x21,
        Key::KeyI => 0x22,
        Key::KeyP => 0x23,
        Key::Return => 0x24,
        Key::KeyL => 0x25,
        Key::KeyJ => 0x26,
        Key::Quote => 0x27,
        Key::KeyK => 0x28,
        Key::SemiColon => 0x29,
        Key::BackSlash => 0x2A,
        Key::Comma => 0x2B,
        Key::Slash => 0x2C,
        Key::KeyN => 0x2D,
        Key::KeyM => 0x2E,
        Key::Dot => 0x2F,
        Key::Tab => 0x30,
        Key::Space => 0x31,
        Key::BackQuote => 0x32,
        Key::Backspace => 0x33,
        Key::Escape => 0x35,
        Key::MetaLeft => 0x37,
        Key::ShiftLeft => 0x38,
        Key::CapsLock => 0x39,
        Key::Alt => 0x3A,
        Key::ControlLeft => 0x3B,
        Key::ShiftRight => 0x3C,
        Key::AltGr => 0x3D,
        Key::ControlRight => 0x3E,
        Key::Function => 0x3F,
        Key::F1 => 0x7A,
        Key::F2 => 0x78,
        Key::F3 => 0x63,
        Key::F4 => 0x76,
        Key::F5 => 0x60,
        Key::F6 => 0x61,
        Key::F7 => 0x62,
        Key::F8 => 0x64,
        Key::F9 => 0x65,
        Key::F10 => 0x6D,
        Key::F11 => 0x67,
        Key::F12 => 0x6F,
        Key::Insert => 0x72,
        Key::Home => 0x73,
        Key::PageUp => 0x74,
        Key::Delete => 0x75,
        Key::End => 0x77,
        Key::PageDown => 0x79,
        Key::LeftArrow => 0x7B,
        Key::RightArrow => 0x7C,
        Key::DownArrow => 0x7D,
        Key::UpArrow => 0x7E,
        _ => return None,
    };
    Some(keycode)
}
