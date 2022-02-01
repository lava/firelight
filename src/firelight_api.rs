// FIXME: move everything else into separate files

use std::sync::mpsc;

use crate::renderer;
use std::os::unix::net::UnixStream;

use crate::control_msg::Control;
use crate::control_msg::ControlMsg;

pub struct Handle {
    thread: Option<std::thread::JoinHandle<()>>,
    tx: mpsc::Sender<ControlMsg>,

    // Remember the last-sent state so we can offer
    // convenience methods to toggle on/off, adjust
    // brightness, etc.
    state: Control,
}

impl Handle {
    pub fn new(socket: UnixStream, strands: Vec<usize>) -> Handle {
        let (tx, rx) = mpsc::channel();
        let join_handle = std::thread::spawn(move || {
            let thread_data = renderer::RenderThreadData {
                rx: rx,
                socket: socket,
                strands: strands,
                state: Control::default(),
            };

            return renderer::render_thread(thread_data);
        });

        return Handle {
            thread: Some(join_handle),
            tx: tx,
            state: Control::default(),
        };
    }

    /// Fully set state.
    pub fn control(&mut self, control: Control) {
        self.state = control;
        let _ = self.tx.send(ControlMsg::External(control));
    }

    // Convenience functions to partially change the state.

    /// Toggle the lamp on/off.
    pub fn toggle(&mut self) {
        let mut toggled = self.state;
        toggled.on = !self.state.on;
        self.control(toggled);
    }

    pub fn set_brightness(&mut self, brightness: u8) {
        let mut adjusted = self.state;
        adjusted.brightness = brightness;
        self.control(adjusted);
    }

    pub fn adjust_brightness(&mut self, brightness_delta: i32) {
        let brightness = if brightness_delta > 0 {
            self.state.brightness.saturating_add(brightness_delta as u8)
        } else {
            self.state
                .brightness
                .saturating_sub(-brightness_delta as u8)
        };
        let mut adjusted = self.state;
        adjusted.brightness = brightness;
        self.control(adjusted);
    }

    // Getters for the current state.

    pub fn brightness(&self) -> u8 {
        return self.state.brightness;
    }

    /// Destructor, joins the render thread.
    pub fn drop(&mut self) {
        let _ = self.tx.send(ControlMsg::Shutdown);
        // This is apparently a standard idiom known as the "option dance" [1]
        // [1]: https://users.rust-lang.org/t/spawn-threads-and-join-in-destructor/1613/9
        if let Some(handle) = self.thread.take() {
            match handle.join() {
                Ok(_) => (),
                Err(err) => println!("error while joining: {:#?}", err),
            }
        }
    }
} // impl handle
