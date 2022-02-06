// FIXME: move everything else into separate files

use std::sync::mpsc;

use anyhow::anyhow;
use crate::renderer;
use std::os::unix::net::UnixStream;

use crate::renderer::RendererCommand;

#[derive(Copy, Clone, Debug)]
pub enum Effect {
    Static,
    Fire,
}

impl Effect {
    pub fn to_string(&self) -> String {
        return match self {
            Effect::Static => "static".to_string(),
            Effect::Fire => "fire".to_string(),
        };
    }

    pub fn from_string(s: &str) -> anyhow::Result<Effect> {
        return match s {
            "static" => Ok(Effect::Static),
            "fire" => Ok(Effect::Fire),
            _ => Err(anyhow!("invalid effect {}", s)),
        }
    }
}

/// Sent by clients.
/// Used to control the state of the renderer.
#[derive(Copy, Clone, Debug)]
pub struct Control {
    /// Whether the lamp is currently on or off.
    pub on: bool,

    /// The currently selected effect.
    pub effect: Effect,

    /// Brightness in [0, 255]
    pub brightness: u8,

    /// Hue in [0.0, 360.0], Saturation in [0.0, 100.0]
    pub color_hs: (f32, f32),
}

impl Control {
    // Also the initial state when booting
    pub fn default() -> Control {
        return Control {
            on: false,
            effect: Effect::Static,
            brightness: 255,
            color_hs: (0.0, 0.0),
        };
    }
}


pub struct Handle {
    thread: Option<std::thread::JoinHandle<()>>,
    tx: mpsc::Sender<RendererCommand>,

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
        let _ = self.tx.send(RendererCommand::ControlMsg(control));
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

    pub fn is_on(&self) -> bool {
        return self.state.on;
    }

    pub fn brightness(&self) -> u8 {
        return self.state.brightness;
    }

    pub fn color_hs(&self) -> (f32, f32) {
        return self.state.color_hs;
    }

    pub fn effect(&self) -> Effect {
        return self.state.effect;
    }

    /// Destructor, joins the render thread.
    pub fn drop(&mut self) {
        let _ = self.tx.send(RendererCommand::Shutdown);
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
