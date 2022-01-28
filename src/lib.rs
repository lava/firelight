extern crate ws281x;

use std::sync::mpsc;

pub struct Control {
    pub on: bool,
    brightness: u8,
    // TODO:
    //color_rgb: (u8, u8, u8);
}

pub struct Handle {
    thread: Option<std::thread::JoinHandle<()>>,
    tx: mpsc::Sender<ControlMsg>,

    // Remember the last-sent state so we can offer
    // convenience methods to toggle on/off, adjust
    // brightness, etc.
    state: Control,
}

enum ControlMsg {
    Shutdown,
    External(Control),
}

struct ControlThreadData {
    rx: mpsc::Receiver<ControlMsg>,

    hw: ws281x::handle::Handle,
    strands: Vec<usize>,
    strands_cum: Vec<usize>,

    // the last received control msg
    state: Control,
}

impl Control {
  fn default() -> Control {
    return Control {on: false, brightness: 255};
  }
}


impl Handle {
    pub fn new(
        ws281x_dma: i32,
        ws281x_channel: usize,
        ws281x_pin: i32,
        strands: Vec<usize>,
    ) -> Handle {
        let (tx, rx) = mpsc::channel();
        let join_handle = std::thread::spawn(move || {
            let mut total = 0;
            let mut cumsum = Vec::new();
            for strand in &strands {
                let prev = cumsum.last().copied().unwrap_or(0);
                cumsum.push(prev + strand);
                total += strand;
            }
            // FIXME: unwrap
            // FIXME: add an option for initial brightness
            let hw_channel = ws281x::channel::new()
                .pin(ws281x_pin)
                .count(total)
                .brightness(55)
                .build()
                .unwrap();
            // FIXME: unwrap
            let handler = ws281x::handle::new()
                .dma(ws281x_dma)
                .channel(ws281x_channel, hw_channel)
                .build()
                .unwrap();

            let control_data = ControlThreadData {
                rx: rx,
                hw: handler,
                strands: strands,
                strands_cum: cumsum,
                state: Control::default(),
            };
            return light_control_thread(control_data);
        });

        return Handle {
            thread: Some(join_handle),
            tx: tx,
            state: Control::default(),
        };
    }

    // fully set state
    pub fn control(&mut self, control: Control) {
        let _ = self.tx.send(ControlMsg::External(control));
    }

    // convenience functions
    pub fn toggle(&mut self) {
        let toggled = Control{on: !self.state.on, brightness: self.state.brightness};
        self.control(toggled);
    }

    pub fn adjust_brightness(&mut self, brightness: u8) {
        let adjusted = Control{on: self.state.on, brightness: brightness};    
        self.control(adjusted);
    }

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

fn light_control_thread(mut data: ControlThreadData) -> () {
    loop {
        let mut idx = 0;

        let colors_on = vec![0xffffff, 0xffffff, 0xffffff, 0xffffff];
        let colors_off = vec![0x0, 0x0, 0x0, 0x0];
        let colors = if data.state.on { colors_on } else { colors_off };
        data.hw.channel_mut(0).set_brightness(data.state.brightness);
        // TODO: support multiple channels
        for (i, led) in data.hw.channel_mut(0).leds_mut().iter_mut().enumerate() {
            while idx < data.strands_cum.len() && i >= data.strands_cum[idx] {
                idx += 1;
            }
            if idx >= colors.len() {
                break;
            }
            *led = colors[idx];
        }

        data.hw.render().unwrap();
        data.hw.wait().unwrap();

        let msg = data.rx.recv().unwrap();
        match msg {
            ControlMsg::Shutdown => break,
            ControlMsg::External(control) => data.state = control,
        }
    }
}
