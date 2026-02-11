#![windows_subsystem = "windows"]

mod serializable_event;
use serializable_event::SerializableEvent;

use iced::widget::{button, checkbox, column, container, row, text, text_input, Column};
use iced::{Alignment, Application, Command, Element, Length, Settings, Theme};
use rdev::{simulate, Event, EventType, Key, SimulateError};
use simplyscriptor2::*;
use std::{
    fs::File,
    io::Read,
    io::Write,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

fn load_icon() -> Option<iced::window::Icon> {
    let icon_bytes = include_bytes!("../resource/icons/simply-scriptor-no-line-256x256.png");

    if let Ok(img) = image::load_from_memory(icon_bytes) {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let icon_data = rgba.into_raw();

        if let Ok(icon) = iced::window::icon::from_rgba(icon_data, width, height) {
            return Some(icon);
        }
    }

    None
}

// Global channel for rdev events
static EVENT_SENDER: once_cell::sync::OnceCell<std::sync::mpsc::Sender<Event>> =
    once_cell::sync::OnceCell::new();

fn main() -> iced::Result {
    // Set up the event channel before anything else
    let (tx, rx) = std::sync::mpsc::channel();
    EVENT_SENDER.set(tx).expect("Failed to set event sender");

    // Main behavior flags, properties, and events vector
    let events = Arc::new(Mutex::new(Vec::new()));
    let record = Arc::new(AtomicBool::new(false));
    let run = Arc::new(AtomicBool::new(false));
    let infinite_loop = Arc::new(AtomicBool::new(true));
    let loop_count = Arc::new(Mutex::new(1));
    let delay = Arc::new(AtomicBool::new(true));
    let halt_actions = Arc::new(AtomicBool::new(false));

    // Clone for the event receiver thread
    let record_clone = Arc::clone(&record);
    let run_clone = Arc::clone(&run);
    let events_clone = Arc::clone(&events);
    let halt_actions_clone = Arc::clone(&halt_actions);

    // Spawn event receiver thread that processes rdev events
    thread::spawn(move || {
        for event in rx.iter() {
            if halt_actions_clone.load(Ordering::Relaxed) {
                continue;
            }

            // Handle keyboard shortcuts
            if event.event_type == EventType::KeyRelease(Key::Comma)
                && !record_clone.load(Ordering::Relaxed)
            {
                record_clone.store(true, Ordering::Relaxed);
                log("Recording...");
                events_clone.lock().unwrap().clear();
                continue;
            }

            if event.event_type == EventType::KeyRelease(Key::Dot)
                && record_clone.load(Ordering::Relaxed)
            {
                record_clone.store(false, Ordering::Relaxed);
                log("Stopped recording...");
                continue;
            }

            if event.event_type == EventType::KeyRelease(Key::Slash) {
                if !run_clone.load(Ordering::Relaxed) && !record_clone.load(Ordering::Relaxed) {
                    log("Running...");
                    run_clone.store(true, Ordering::Relaxed);
                } else if run_clone.load(Ordering::Relaxed) {
                    log("Stopped running...");
                    run_clone.store(false, Ordering::Relaxed);
                }
                continue;
            }

            // Record events
            if record_clone.load(Ordering::Relaxed) && !run_clone.load(Ordering::Relaxed) {
                events_clone.lock().unwrap().push(event);
            }
        }
    });

    let run_ref = Arc::clone(&run);
    let events_ref = Arc::clone(&events);
    let infinite_loop_ref = Arc::clone(&infinite_loop);
    let loop_count_ref = Arc::clone(&loop_count);
    let delay_ref = Arc::clone(&delay);

    thread::spawn(move || {
        event_loop(
            events_ref,
            run_ref,
            infinite_loop_ref,
            loop_count_ref,
            delay_ref,
        );
    });

    // Start rdev listener in a separate thread
    // On macOS, use grab() for better trackpad click detection
    thread::spawn(|| {
        #[cfg(target_os = "macos")]
        {
            if let Err(error) = rdev::grab(move |event| {
                rdev_callback(event.clone());
                // Return None to pass the event through to the system
                // Return Some(event) to block the event
                None
            }) {
                eprintln!("rdev grab error: {:?}", error);
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            if let Err(error) = rdev::listen(rdev_callback) {
                eprintln!("rdev listen error: {:?}", error);
            }
        }
    });

    ScriptorApp::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(200.0, 270.0),
            resizable: false,
            icon: load_icon(),
            decorations: true,
            max_size: Some(iced::Size::new(200.0, 270.0)),
            ..Default::default()
        },
        flags: AppFlags {
            events,
            record,
            run,
            infinite_loop,
            loop_count,
            delay,
            halt_actions,
        },
        ..Settings::default()
    })
}

// Callback function for rdev events
fn rdev_callback(event: Event) {
    if let Some(sender) = EVENT_SENDER.get() {
        let _ = sender.send(event);
    }
}

#[derive(Default)]
struct AppFlags {
    events: Arc<Mutex<Vec<Event>>>,
    record: Arc<AtomicBool>,
    run: Arc<AtomicBool>,
    infinite_loop: Arc<AtomicBool>,
    loop_count: Arc<Mutex<i32>>,
    delay: Arc<AtomicBool>,
    halt_actions: Arc<AtomicBool>,
}

struct ScriptorApp {
    events: Arc<Mutex<Vec<Event>>>,
    record: Arc<AtomicBool>,
    run: Arc<AtomicBool>,
    infinite_loop: Arc<AtomicBool>,
    loop_count: Arc<Mutex<i32>>,
    delay: Arc<AtomicBool>,
    halt_actions: Arc<AtomicBool>,
    script_file_name: String,
    minimize_on_action: bool,
    infinite_loop_checked: bool,
    delay_checked: bool,
    loop_count_value: i32,
    was_recording: bool,
    was_running: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Message {
    Record,
    StopRecording,
    Run,
    Open,
    Save,
    InfiniteLoopToggled(bool),
    DelayToggled(bool),
    MinimizeToggled(bool),
    LoopCountChanged(i32),
    LoopCountInputChanged(String),
    FileOpened(Option<std::path::PathBuf>),
    FileSaved(Option<std::path::PathBuf>),
    Tick,
}

impl Application for ScriptorApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = AppFlags;

    fn new(flags: AppFlags) -> (Self, Command<Message>) {
        (
            ScriptorApp {
                events: flags.events,
                record: flags.record,
                run: flags.run,
                infinite_loop: flags.infinite_loop,
                loop_count: flags.loop_count,
                delay: flags.delay,
                halt_actions: flags.halt_actions,
                script_file_name: String::new(),
                minimize_on_action: false,
                infinite_loop_checked: true,
                delay_checked: true,
                loop_count_value: 1,
                was_recording: false,
                was_running: false,
            },
            Command::perform(async {}, |_| Message::Tick),
        )
    }

    fn title(&self) -> String {
        String::new()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Record => {
                if !self.record.load(Ordering::Relaxed) {
                    self.script_file_name = String::new();
                    log("Recording...");
                    self.record.store(true, Ordering::Relaxed);
                    self.events.lock().unwrap().clear();

                    if self.minimize_on_action {
                        return iced::window::minimize(iced::window::Id::MAIN, true);
                    }
                }
                Command::none()
            }
            Message::StopRecording => {
                if self.record.load(Ordering::Relaxed) {
                    log("Stopped recording...");
                    self.record.store(false, Ordering::Relaxed);
                }
                Command::none()
            }
            Message::Run => {
                if !self.run.load(Ordering::Relaxed) {
                    log("Running...");
                    self.run.store(true, Ordering::Relaxed);

                    if self.minimize_on_action {
                        return iced::window::minimize(iced::window::Id::MAIN, true);
                    }
                } else {
                    log("Stopped running...");
                    self.run.store(false, Ordering::Relaxed);
                }
                Command::none()
            }
            Message::Open => Command::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Script Files", &["bin"])
                        .pick_file()
                        .await
                        .map(|f| f.path().to_path_buf())
                },
                Message::FileOpened,
            ),
            Message::Save => Command::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Script Files", &["bin"])
                        .save_file()
                        .await
                        .map(|f| f.path().to_path_buf())
                },
                Message::FileSaved,
            ),
            Message::FileOpened(path) => {
                if let Some(path) = path {
                    self.halt_actions.store(true, Ordering::Relaxed);

                    match File::open(&path) {
                        Ok(mut file) => {
                            let mut buffer = Vec::<u8>::new();
                            if file.read_to_end(&mut buffer).is_ok() {
                                // Deserialize as SerializableEvent and convert to Event
                                if let Ok(decoded) =
                                    bincode::deserialize::<Vec<SerializableEvent>>(&buffer)
                                {
                                    let events_converted: Vec<Event> =
                                        decoded.into_iter().map(|e| e.into()).collect();

                                    let mut events = self.events.lock().unwrap();
                                    events.clear();
                                    *events = events_converted;

                                    let file_name =
                                        path.file_name().unwrap().to_str().unwrap().to_string();

                                    if file_name.len() > 12 {
                                        self.script_file_name = format!("{}...", &file_name[0..12]);
                                    } else {
                                        self.script_file_name = file_name;
                                    }
                                } else {
                                    log("Error: Could not deserialize file");
                                }
                            } else {
                                log("Error: Could not read file");
                            }
                        }
                        Err(e) => {
                            log(&format!("Error opening file: {}", e));
                        }
                    }

                    self.halt_actions.store(false, Ordering::Relaxed);
                }
                Command::none()
            }
            Message::FileSaved(path) => {
                if let Some(mut path) = path {
                    self.halt_actions.store(true, Ordering::Relaxed);

                    if path.extension().is_none() {
                        path.set_extension("bin");
                    }

                    match File::create(&path) {
                        Ok(mut file) => {
                            let events = self.events.lock().unwrap();
                            let serializable: Vec<SerializableEvent> =
                                events.iter().map(|e| e.clone().into()).collect();
                            let encoded: Vec<u8> = bincode::serialize(&serializable).unwrap();

                            if file.write_all(&encoded).is_ok() {
                                let file_name =
                                    path.file_name().unwrap().to_str().unwrap().to_string();

                                if file_name.len() > 12 {
                                    self.script_file_name = format!("{}...", &file_name[0..12]);
                                } else {
                                    self.script_file_name = file_name;
                                }
                                log("File saved successfully");
                            } else {
                                log("Error: Could not write to file");
                            }
                        }
                        Err(e) => {
                            log(&format!("Error creating file: {}", e));
                        }
                    }

                    self.halt_actions.store(false, Ordering::Relaxed);
                }
                Command::none()
            }
            Message::InfiniteLoopToggled(value) => {
                self.infinite_loop_checked = value;
                self.infinite_loop.store(value, Ordering::Relaxed);
                Command::none()
            }
            Message::DelayToggled(value) => {
                self.delay_checked = value;
                self.delay.store(value, Ordering::Relaxed);
                Command::none()
            }
            Message::MinimizeToggled(value) => {
                self.minimize_on_action = value;
                Command::none()
            }
            Message::LoopCountChanged(value) => {
                self.loop_count_value = value;
                let mut loop_count = self.loop_count.lock().unwrap();
                *loop_count = value;
                Command::none()
            }
            Message::LoopCountInputChanged(input) => {
                if let Ok(value) = input.parse::<i32>() {
                    if value >= 1 {
                        self.loop_count_value = value;
                        let mut loop_count = self.loop_count.lock().unwrap();
                        *loop_count = value;
                    }
                }
                Command::none()
            }
            Message::Tick => {
                let is_recording = self.record.load(Ordering::Relaxed);
                let is_running = self.run.load(Ordering::Relaxed);

                if is_recording && !self.was_recording && self.minimize_on_action {
                    self.script_file_name = String::new();
                    self.was_recording = is_recording;
                    self.was_running = is_running;

                    return Command::batch(vec![
                        iced::window::minimize(iced::window::Id::MAIN, true),
                        Command::perform(
                            async {
                                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            },
                            |_| Message::Tick,
                        ),
                    ]);
                }

                if is_running && !self.was_running && self.minimize_on_action {
                    self.was_recording = is_recording;
                    self.was_running = is_running;

                    return Command::batch(vec![
                        iced::window::minimize(iced::window::Id::MAIN, true),
                        Command::perform(
                            async {
                                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            },
                            |_| Message::Tick,
                        ),
                    ]);
                }

                if is_recording && !self.was_recording {
                    self.script_file_name = String::new();
                }

                self.was_recording = is_recording;
                self.was_running = is_running;

                Command::perform(
                    async {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    },
                    |_| Message::Tick,
                )
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let script_label = if !self.script_file_name.is_empty() {
            &self.script_file_name
        } else {
            ""
        };

        let file_section = row![text("Script:").size(12), text(script_label).size(12),]
            .spacing(5)
            .align_items(Alignment::Center);

        let open_button = button(
            text("Open")
                .size(12)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
        )
        .on_press(Message::Open)
        .width(Length::Fixed(184.0))
        .padding(6);
        let save_button = button(
            text("Save")
                .size(12)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
        )
        .on_press(Message::Save)
        .width(Length::Fixed(184.0))
        .padding(6);

        let minimize_checkbox = checkbox("Minimize", self.minimize_on_action)
            .on_toggle(Message::MinimizeToggled)
            .size(14)
            .text_size(12);

        let delay_checkbox = checkbox("Natural Delay", self.delay_checked)
            .on_toggle(Message::DelayToggled)
            .size(14)
            .text_size(12);

        let infinite_checkbox = checkbox("Infinite Loop", self.infinite_loop_checked)
            .on_toggle(Message::InfiniteLoopToggled)
            .size(14)
            .text_size(12);

        let checkboxes = column![minimize_checkbox, delay_checkbox, infinite_checkbox,]
            .spacing(2)
            .align_items(Alignment::Start);

        let loop_count_label = text("Loop Count:").size(12);

        let loop_minus = button(
            container(
                text("-")
                    .size(12)
                    .horizontal_alignment(iced::alignment::Horizontal::Center)
                    .vertical_alignment(iced::alignment::Vertical::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y(),
        )
        .on_press(Message::LoopCountChanged(
            (self.loop_count_value - 1).max(1),
        ))
        .width(Length::Fixed(20.0))
        .height(Length::Fixed(20.0))
        .padding(0);

        let loop_input = text_input("1", &self.loop_count_value.to_string())
            .on_input(Message::LoopCountInputChanged)
            .width(70)
            .size(12)
            .padding([2, 5]);

        let loop_plus = button(
            container(
                text("+")
                    .size(12)
                    .horizontal_alignment(iced::alignment::Horizontal::Center)
                    .vertical_alignment(iced::alignment::Vertical::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y(),
        )
        .on_press(Message::LoopCountChanged(self.loop_count_value + 1))
        .width(Length::Fixed(20.0))
        .height(Length::Fixed(20.0))
        .padding(0);

        let loop_count_row = row![loop_count_label, loop_minus, loop_input, loop_plus]
            .spacing(3)
            .align_items(Alignment::Center);

        let record_button = button(
            text("Record [ , ]")
                .size(12)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
        )
        .on_press(Message::Record)
        .width(Length::Fixed(184.0))
        .padding(6);
        let stop_button = button(
            text("Stop Recording [ . ]")
                .size(12)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
        )
        .on_press(Message::StopRecording)
        .width(Length::Fixed(184.0))
        .padding(6);
        let run_button = button(
            text("Run [ / ]")
                .size(12)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
        )
        .on_press(Message::Run)
        .width(Length::Fixed(184.0))
        .padding(6);

        let content: Column<Message> = column![
            file_section,
            open_button,
            save_button,
            container(checkboxes).width(Length::Fill).center_x(),
            loop_count_row,
            record_button,
            stop_button,
            run_button,
        ]
        .spacing(4)
        .padding([6, 8, 6, 8]);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::custom(
            String::from("Dark Brown"),
            iced::theme::Palette {
                background: iced::Color::from_rgb(
                    0x26 as f32 / 255.0,
                    0x1b as f32 / 255.0,
                    0x0e as f32 / 255.0,
                ),
                text: iced::Color::WHITE,
                primary: iced::Color::from_rgb(
                    0x26 as f32 / 255.0,
                    0x1b as f32 / 255.0,
                    0x0e as f32 / 255.0,
                ),
                success: iced::Color::from_rgb(
                    0x26 as f32 / 255.0,
                    0x1b as f32 / 255.0,
                    0x0e as f32 / 255.0,
                ),
                danger: iced::Color::from_rgb(0.8, 0.2, 0.2),
            },
        )
    }
}

fn event_loop(
    events: Arc<Mutex<Vec<Event>>>,
    run: Arc<AtomicBool>,
    infinite_loop: Arc<AtomicBool>,
    loop_count: Arc<Mutex<i32>>,
    delay: Arc<AtomicBool>,
) {
    loop {
        if run.load(Ordering::Relaxed) {
            let events_ref = Arc::clone(&events);
            let run_ref = Arc::clone(&run);
            let infinite_loop_ref = Arc::clone(&infinite_loop);
            let loop_count_ref = Arc::clone(&loop_count);
            let delay_ref = Arc::clone(&delay);
            send_events(
                events_ref,
                run_ref,
                infinite_loop_ref,
                loop_count_ref,
                delay_ref,
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn send_events(
    events: Arc<Mutex<Vec<Event>>>,
    run: Arc<AtomicBool>,
    infinite_loop: Arc<AtomicBool>,
    loop_count: Arc<Mutex<i32>>,
    delay: Arc<AtomicBool>,
) {
    let events = events.lock().unwrap().to_vec();
    if events.is_empty() {
        log("There aren't any events to run!");
        run.store(false, Ordering::Relaxed);
        return;
    }

    let loop_count = loop_count.lock().unwrap();
    let mut i = 0;
    while i < *loop_count {
        let start_time = std::time::Instant::now();
        let recording_start = events[0].time;

        let mut halted = false;
        for event in &events {
            if !run.load(Ordering::Relaxed) {
                log("Running halted!");
                halted = true;
                break;
            }

            if delay.load(Ordering::Relaxed) {
                let target_offset = event.time.duration_since(recording_start).unwrap();
                let current_offset = start_time.elapsed();

                if target_offset > current_offset {
                    let sleep_duration = target_offset - current_offset;
                    spin_sleep::sleep(sleep_duration);
                }
            } else {
                spin_sleep::sleep(Duration::from_micros(50));
            }

            send_event(&event.event_type);
        }

        if halted {
            break;
        }

        match simulate(&EventType::KeyRelease(Key::Dot)) {
            Ok(()) => (),
            Err(SimulateError) => {
                eprintln!("Could not send final release key.");
            }
        };

        if !infinite_loop.load(Ordering::Relaxed) {
            i += 1;
        }
    }

    run.store(false, Ordering::Relaxed);
    log("Done");
}
