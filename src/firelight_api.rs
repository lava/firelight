// FIXME: move everything else into separate files

use std::sync::mpsc;

use crate::renderer;

use crate::control_msg::ControlMsg;
use crate::control_msg::Control;

pub struct Handle {
    thread: Option<std::thread::JoinHandle<()>>,
    tx: mpsc::Sender<ControlMsg>,

    // Remember the last-sent state so we can offer
    // convenience methods to toggle on/off, adjust
    // brightness, etc.
    state: Control,
}

impl Handle {
    pub fn new(rpi_dma: i32, rpi_channel: usize, rpi_pin: i32, strands: Vec<usize>) -> Handle {
        let (tx, rx) = mpsc::channel();
        let join_handle = std::thread::spawn(move || {
            let total = strands.iter().sum();
            // The `rust-ws2811x` library has a built-in `brightness` parameter,
            // that's used to scale every color channel by `c = c * (brightness+1) / 256`.
            // We don't expose that to the user and instead set it to 255 to pass
            // through the exact rgb values that we put in, letting the user take
            // care of handling color spaces, brightness etc.
            let hw_channel = ws281x::channel::new()
                .pin(rpi_pin)
                .count(total)
                .brightness(255)
                .build()
                .unwrap();

            // FIXME: move handler/channel creation into
            // a separate `lightingd` binary.
            let handler = ws281x::handle::new()
                .dma(rpi_dma)
                .channel(rpi_channel, hw_channel)
                .build()
                .unwrap();

            let control_data = renderer::ControlThreadData {
                rx: rx,
                hw: handler,
                strands: strands,
                state: Control::default(),
            };
            return renderer::light_control_thread(control_data);
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

    /// Destructor, joins the control thread.
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
