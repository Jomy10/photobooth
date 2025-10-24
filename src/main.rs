#![feature(mpmc_channel)]

use std::cell::RefCell;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

use anyhow::{anyhow, Result};
use chrono::{TimeDelta, Utc};
use drm::buffer::DrmFourcc;
use image::{ImageBuffer, ImageFormat};
use log::*;
use photobooth::camera::{Camera, CameraManager};
use photobooth::display::Display;
use photobooth::input::InputManager;
use photobooth::ui::{TextBox, UIElement, UI};
use photobooth::utils::UnsafePtr;

#[derive(Debug, Clone, Copy, PartialEq)]
enum AppState {
    TakePicturePrompt,
    /// Shows the countdown and previeuw
    TakingPicture,
    /// Capture
    TakePicture,
}

impl AppState {
    fn show_video_stream(&self) -> bool {
        return *self == AppState::TakingPicture;
    }
}

struct App<'a> {
    config: photobooth::config::Config,
    disp: Display,
    camera: Camera<'a>,
    camera_receiver: std::sync::mpmc::Receiver<libcamera::request::Request>,
    ui: UI,
    #[allow(unused)]
    input: InputManager,
    time_sensitive_ui: Vec<(Rc<RefCell<TextBox>>, TimeDelta, Box<dyn Fn(TimeDelta, TimeDelta, &Rc<RefCell<TextBox>>) -> ()>)>,
    state_change_receiver: Receiver<AppState>,
    state_change_sender: Sender<AppState>,

    state: AppState
}

impl<'a> App<'a> {
    pub fn new(
        config: photobooth::config::Config,
        cam_manager: &'a CameraManager
    ) -> Result<Self> {
        let format_drm = DrmFourcc::Xrgb8888;
        let format_u32 = u32::from_le_bytes([b'X', b'R', b'2', b'4']);

        // Display
        info!("Initializing DRM (display)");
        let disp = Display::new("/dev/dri/by-path/platform-gpu-card", format_drm, 24, 32)?;

        info!("Initialized display {}x{}", disp.size().0, disp.size().1);

        // Camera
        info!("Initializing camera");
        let mut camera = Camera::new(&cam_manager, format_u32)?;

        // camera.queue_video_requests()?;
        let camera_receiver = camera.on_request_receiver();

        // Ui
        let (mut ui, touch_sender) = UI::new((disp.size().0 as usize, disp.size().1 as usize));
        ui.set_text_color(config.text_color);

        // Input
        let input = InputManager::new(
            "/dev/input/by-id/usb-QDtech_MPI7003-event-if00".to_string(), // TODO: parameter
            disp.size().0 as u32, disp.size().1 as u32
        );
        input.subscribe(touch_sender);

        let (state_change_sender, state_change_receiver) = std::sync::mpsc::channel();

        return Ok(App {
            config,
            disp,
            camera,
            camera_receiver,
            ui,
            input,
            state: AppState::TakePicturePrompt,
            time_sensitive_ui: Vec::new(),
            state_change_receiver,
            state_change_sender,
        });
    }

    pub fn run(&mut self) -> Result<()> {
        self.transition(None, self.state)?;

        let mut req: Option<libcamera::request::Request> = None;

        let mut prev_time = Utc::now();
        loop {
            let new_time = Utc::now();
            let delta = new_time - prev_time;
            // Show video stream or clear screen
            let show_video_stream = self.state.show_video_stream();
            if show_video_stream {
                req = Some(self.camera_receiver.recv_timeout(Duration::from_secs(2))?);

                let fb_ptr = self.camera.video_stream().get_mapped_buffer(req.as_ref().unwrap().cookie());
                unsafe { self.disp.copy_dma_buf(fb_ptr, self.camera.video_stream().get_frame_size() as usize)? };
            } else {
                self.disp.clear(self.config.bg_color)?;
            }

            // Update UI
            for (textbox, ui_delta, cb) in self.time_sensitive_ui.iter_mut() {
                cb(*ui_delta, *ui_delta + delta, textbox);
                *ui_delta = *ui_delta + delta;
            }
            self.ui.update();

            // Render UI
            {
                let mut buffer = self.disp.back_buffer_mut()?;
                let buffer = buffer.as_mut();
                self.ui.render(buffer);
            }

            // Update display
            self.disp.swap_buffers()?;

            // Reuse video stream request
            if show_video_stream {
                let req = req.take().unwrap();
                self.camera.resubmit_stream_request(req)?;
                // req.reuse(ReuseFlag::REUSE_BUFFERS);
                // self.camera.queue_video_request(req)?;
            }

            // Transition state
            if let Ok(new_state) = self.state_change_receiver.try_recv() {
                self.transition(Some(self.state), new_state)?;
            }

            prev_time = new_time;
        }
    }

    pub fn transition(&mut self, previous_state: Option<AppState>, state: AppState) -> Result<()> {
        self.state = state;
        self.time_sensitive_ui.clear();
        self.ui.clear();

        match previous_state {
            Some(AppState::TakingPicture) => {
                self.camera.stop_stream()?;
            },
            Some(_) | None => {},
        }

        match state {
            AppState::TakePicturePrompt => {
                let textbox = self.ui.add_text_box(
                    (0., 0.),
                    (self.disp.size().0 as f32, self.disp.size().1 as f32),
                    fontdue::layout::HorizontalAlign::Center,
                    fontdue::layout::VerticalAlign::Middle
                );
                let mut textbox = textbox.borrow_mut();
                textbox.add_text(&self.config.take_picture_text, self.config.text_size);
                let sender = self.state_change_sender.clone();
                textbox.add_touch_listener(Box::new(move || {
                    sender.send(AppState::TakingPicture).unwrap();
                }));
            },
            AppState::TakingPicture => {
                self.camera.start_stream()?;
                let textbox = self.ui.add_text_box(
                    (0., 0.),
                    (self.disp.size().0 as f32, self.disp.size().1 as f32),
                    fontdue::layout::HorizontalAlign::Center,
                    fontdue::layout::VerticalAlign::Middle
                );
                {
                    let mut tb = textbox.borrow_mut();
                    tb.add_text(format!("{}", self.config.countdown), self.config.countdown_text_size);
                }

                let countdown = self.config.countdown;
                let countdown_text_size = self.config.countdown_text_size;
                let sender = self.state_change_sender.clone();
                self.time_sensitive_ui.push((textbox, TimeDelta::zero(), Box::new(move |prev_delta, delta, textbox| {
                    if prev_delta.num_seconds() != delta.num_seconds() {
                        let mut textbox = textbox.borrow_mut();
                        textbox.clear();
                        textbox.add_text(format!("{}", countdown - delta.num_seconds() as u32), countdown_text_size);

                        if countdown - delta.num_seconds() as u32 == 1 {
                            sender.send(AppState::TakePicture).unwrap();
                        }
                    }
                })))
            },
            AppState::TakePicture => {
                let file_name = "test.png";
                let file = File::create(file_name)?;
                let mut writer = BufWriter::new(file);
                let (capture_sender, capture_waiter) = std::sync::mpsc::channel();
                let (image_sender, image_waiter) = std::sync::mpsc::channel();
                let (signal_continue, waiter) = std::sync::mpsc::channel();

                let camera: *mut Camera = &mut self.camera;
                let camera: UnsafePtr<Camera<'static>> = unsafe { UnsafePtr { ptr: std::mem::transmute(camera) } }; // safe because we don't leave this function
                let camera_thread_handle: JoinHandle<Result<()>> = std::thread::spawn(move || {
                    let camera: &mut Camera = unsafe { camera.as_mut() };
                    camera.capture(
                        &mut writer,
                        ImageFormat::Png,
                        Some(capture_sender),
                        Some(image_sender),
                        Some(waiter)
                    )
                });

                {
                    let mut back_buffer = self.disp.back_buffer_mut()?;
                    let buffer = back_buffer.as_mut();
                    buffer.fill(0xFF); // fill white
                }

                // White out screen when image captured
                capture_waiter.recv()?;
                self.disp.swap_buffers()?;

                // Get image and show on screen
                let size = (self.disp.size().0 as u32, self.disp.size().1 as u32);
                let image_processing_thread_handle: JoinHandle<Result<ImageBuffer<image::Rgba<u8>, Vec<u8>>>> = std::thread::spawn(move || {
                    let image = image_waiter.recv()?;
                    // Nearest is fastest, but ugliest, Triangle is another good option, but slower
                    let resized_image = image::imageops::resize(image.as_ref(), size.0, size.1, image::imageops::FilterType::Nearest);
                    // let image_buffer = unsafe { std::slice::from_raw_parts(resized_image.as_ptr(), resized_image.len()) };
                    return Ok(resized_image);
                });

                std::thread::sleep(std::time::Duration::from_millis(500));

                // Display done message
                {
                    self.disp.clear(self.config.bg_color)?;

                    let mut back_buffer = self.disp.back_buffer_mut()?;
                    let buffer = back_buffer.as_mut();

                    let textbox = self.ui.add_text_box(
                        (0., 0.),
                        (size.0 as f32, size.1 as f32),
                        fontdue::layout::HorizontalAlign::Center,
                        fontdue::layout::VerticalAlign::Middle
                    );
                    textbox.borrow_mut().add_text(&self.config.done_sentences[rand::random_range(0..self.config.done_sentences.len())], self.config.text_size);
                    self.ui.render(buffer);
                }

                self.disp.swap_buffers()?;

                let t = Utc::now();
                let resized_image = image_processing_thread_handle.join().map_err(|err| anyhow!("{:?}", err))??;

                let dt = t - Utc::now();
                let sleep_time = std::time::Duration::from_secs(self.config.done_show_time as u64);
                let sleep_time = (sleep_time.as_millis() as u64).saturating_sub(dt.num_milliseconds() as u64);
                let sleep_time = std::time::Duration::from_millis(sleep_time);
                std::thread::sleep(sleep_time);

                // Show image
                {
                    let mut disp_buffer = self.disp.back_buffer_mut()?;
                    disp_buffer.copy_from_slice(&resized_image);
                }

                self.disp.swap_buffers()?;

                signal_continue.send(())?;

                std::thread::sleep(std::time::Duration::from_secs(self.config.show_image_time as u64));

                camera_thread_handle.join().map_err(|err| anyhow!("{:?}", err))??;

                self.state_change_sender.send(AppState::TakePicturePrompt)?;
            },
        }

        Ok(())
    }
}

fn configure_logging() -> Result<()> {
    let stdout_log = log4rs::append::console::ConsoleAppender::builder()
        .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new("{d(%H:%M:%S)} {h({l})}: {m}\n")))
        .build();

    let logfile = match log4rs::append::rolling_file::RollingFileAppender::builder()
        .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new("{d} {M}::{f}:{L} {l}: {m}\n")))
        .build(
            "/var/log/photobooth.log",
            Box::new(log4rs::append::rolling_file::policy::compound::CompoundPolicy::new(
                Box::new(log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger::new(10 * 1024 * 1024)), // 10 MB
                Box::new(log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller::builder().build("/var/log/photobooth.{}.log", 5).unwrap()) // keep 5 old files
            ))
        ) {
            Ok(v) => Some(v),
            Err(err) => match err.kind() {
                io::ErrorKind::PermissionDenied => {
                    eprintln!("Could not access logfile. Create it and correct the permissions.");
                    None
                },
                _ => None
            }
        };

    let mut config = log4rs::config::Config::builder()
        .appender(log4rs::config::Appender::builder().build("stdout", Box::new(stdout_log)));
    if let Some(logfile) = logfile {
        config = config.appender(log4rs::config::Appender::builder().build("logfile", Box::new(logfile)));
    }
    let root = log4rs::config::Root::builder()
            .appenders(["logfile", "stdout"]);
    #[cfg(not(debug_assertions))]
    let root = root.build(LevelFilter::Info);
    #[cfg(debug_assertions)]
    let root = root.build(LevelFilter::Trace);
    let config = config.build(root)?;

    _ = log4rs::init_config(config)?;

    info!("Photobooth logging initialized");

    Ok(())
}

fn get_config() -> Result<photobooth::config::Config> {
    let configuration_path = std::env::var_os("PH_CONFIG").unwrap_or("config.yaml".into());
    info!("Reading configuration from {}", configuration_path.to_string_lossy());
    let config: photobooth::config::Config = if std::fs::exists(&configuration_path)? {
        let config_file = File::open(configuration_path)?;
        let config_reader = BufReader::new(config_file);
        serde_yaml::from_reader(config_reader)?
    } else {
        info!("Configuration file not found, using default");
        Default::default()
    };

    info!("Config: {:#?}", config);

    return Ok(config);
}

fn main() -> Result<()> {
    configure_logging()?;

    let config = get_config()?;

    let camera_manager = CameraManager::acquire()?;

    let mut app = App::new(config, &camera_manager)?;
    app.run()?;

    Ok(())
}
