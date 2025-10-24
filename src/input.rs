use std::fs::{File, OpenOptions};
use std::os::fd::OwnedFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::mpsc::{TryRecvError, Sender};
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;

use input::event::touch::TouchEventPosition;
use input::event::TouchEvent;
use input::{Event, Libinput, LibinputInterface};
use libc::{O_RDONLY, O_RDWR, O_WRONLY};
use log::*;

struct Interface;

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> std::result::Result<OwnedFd, i32> {
        OpenOptions::new()
            .custom_flags(flags)
            .read((flags & O_RDONLY != 0) | (flags & O_RDWR != 0))
            .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
            .open(path)
            .map(|file| file.into())
            .map_err(|err| err.raw_os_error().unwrap())
    }
    fn close_restricted(&mut self, fd: OwnedFd) {
        drop(File::from(fd))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TouchInputEvent {
    pub x: f64,
    pub y: f64
}

// #[derive(Debug, Clone)]
// pub enum TouchInputEvent {
//     Click(f64, f64),
//     // MOTION
// }

pub struct InputManager {
    dispatch_thread: Option<JoinHandle<()>>,
    stop_thread: Sender<()>,
    touch_subscribers: Arc<RwLock<Vec<Sender<TouchInputEvent>>>>,
}

impl InputManager {
    pub fn new(path: String, screen_width: u32, screen_height: u32) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let subscribers = Arc::new(RwLock::new(Vec::<Sender<TouchInputEvent>>::new()));

        let subscribers2 = subscribers.clone();
        let handle = std::thread::spawn(move || {
            let mut input = Libinput::new_from_path(Interface);
            input.path_add_device(&path).unwrap();

            let mut touch_start: Option<(f64, f64)> = None;
            // motion: sequence of positions

            'dispatch_loop: loop {
                input.dispatch().unwrap();
                for event in &mut input {
                    match event {
                        Event::Touch(touch_event) => {
                            match touch_event {
                                TouchEvent::Down(touch_down_event) => {
                                    let x = touch_down_event.x_transformed(screen_width);
                                    let y = touch_down_event.y_transformed(screen_height);
                                    trace!("Touch down ({}, {})", x, y);
                                    touch_start = Some((x, y));
                                },
                                TouchEvent::Up(_) => {
                                    let touch = touch_start.unwrap_or((0., 0.));
                                    trace!("Touch up {:?}", touch);
                                    let subscribers = (*subscribers2).read().unwrap();
                                    for subscriber in subscribers.iter() {
                                        subscriber.send(TouchInputEvent{ x: touch.0, y: touch.1 }).unwrap();
                                    }
                                },
                                TouchEvent::Motion(_) => {},
                                TouchEvent::Cancel(_) => {
                                    info!("Unhandled cancel event");
                                },
                                TouchEvent::Frame(_) => {}, // end of an event
                                ev => {
                                    info!("Unhandled touch event {:#?}", ev);
                                },
                            }
                        },
                        _ => {}
                    }
                }

                match rx.try_recv() {
                    Ok(()) => break 'dispatch_loop,
                    Err(TryRecvError::Disconnected) => panic!("sender disconnected in input dispatch thread"),
                    Err(TryRecvError::Empty) => {
                        ::std::thread::sleep(::std::time::Duration::from_millis(10));
                        continue
                    },
                }
            }
        });

        return InputManager {
            // input: input2,
            dispatch_thread: Some(handle),
            stop_thread: tx,
            touch_subscribers: subscribers,
        };
    }

    // Subscribe to touch events
    pub fn subscribe(&self, receiver: Sender<TouchInputEvent>) {
        let mut guard = self.touch_subscribers.write().unwrap();
        guard.push(receiver);
    }
}

impl Drop for InputManager {
    fn drop(&mut self) {
        self.stop_thread.send(()).unwrap();
        self.dispatch_thread.take().unwrap().join().unwrap();
    }
}
