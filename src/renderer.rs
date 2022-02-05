// TODO: re-evaluate pub

use noise::NoiseFn;
use noise::Perlin;
use std::io::Write;
use std::sync::mpsc;

use std::os::unix::net::UnixStream;
use std::time::Duration;

use palette::FromColor;
use palette::Pixel;

use crate::firelight_api::Control;
use crate::firelight_api::Effect;
use crate::daemon;


// TODO: re-evaluate publicness when introducing daemon
pub(crate) enum RendererCommand {
    Shutdown,
    ControlMsg(Control),
}

pub(crate) struct RenderThreadData {
    pub rx: mpsc::Receiver<RendererCommand>,

    pub socket: UnixStream,
    pub strands: Vec<usize>,

    // the last received control msg
    pub state: Control,
}

pub(crate) fn render_thread(mut data: RenderThreadData) -> () {
    let mut t = 0.0;
    let delta = 0.01;
    loop {
        t += delta;
        let color_hsl = palette::Hsl::new(data.state.color_hs.0, data.state.color_hs.1 / 100., data.state.brightness as f32 / 255.);
        let color_rgb = palette::Srgb::from_color(color_hsl);
        let colors = match data.state.effect {
            Effect::Static => render_static(t, color_rgb, &data.strands),
            Effect::Fire => render_fire(t, color_rgb, &data.strands),
        };
        let mut out = Vec::new();
        for original_color in colors {
            let color = if data.state.on {
                original_color.to_u32_rgb()
            } else {
                0
            };
            out.push(color);
        }
        let _ = data.socket.write(daemon::as_bytes(&mut out[..]));

        // TODO: Use a separate timer thread for a stable clock pulse
        let msg = data.rx.recv_timeout(Duration::from_millis(1000 / 60));
        match msg {
            Ok(RendererCommand::Shutdown) => break,
            Ok(RendererCommand::ControlMsg(control)) => {
                data.state = control;
                let color_hsl = palette::Hsv::new(data.state.color_hs.0, data.state.color_hs.1 / 100., data.state.brightness as f32 / 255.);
                println!("color: {:?}/{}/{}", color_hsl.hue, color_hsl.saturation, color_hsl.value);
                let color_rgb = palette::Srgb::from_color(color_hsl);
                println!("color: {}/{}/{}", color_rgb.red, color_rgb.green, color_rgb.blue);
                let rgb : [u8; 3] = color_rgb.into_format().into_raw();
                println!("color: {}/{}/{}", rgb[0], rgb[1], rgb[2]);
            },
            Err(_) => continue,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LedColor {
    // r, g, b
    data: [u8; 3],
}

impl LedColor {
    // Initialize from a u32 that looks like 0x00RRGGBB.
    fn from_u32_rgb(x: u32) -> LedColor {
        return LedColor {
            data: [
                ((x >> 16) & 0xff) as u8,
                ((x >> 8) & 0xff) as u8,
                (x & 0xff) as u8,
            ],
        };
    }

    // Initialize from an u8 array [0xRR, 0xGG, 0xBB]
    fn from_u8_rgb(x: [u8; 3]) -> LedColor {
        return LedColor {
            data: x,
        }
    }

    // Render as 0x00RRGGBB.
    fn to_u32_rgb(&self) -> u32 {
        return ((self.data[0] as u32) << 16)
            | ((self.data[1] as u32) << 8)
            | (self.data[2] as u32);
    }
}

fn render_fire(t: f64, color_rgb: palette::Srgb, strands: &Vec<usize>) -> Vec<LedColor> {
    let perlin = Perlin::default();
    let mut noise = Vec::new();
    for (i, _) in strands.iter().enumerate() {
        // The `perlin.get()` function returns values in [-1, 1].
        // Same noise for all strands.
        //noise.push((perlin.get([t, 0.0]) + 1.0) / 2.0);
        // Independent noise for all strands.
        noise.push((perlin.get([t, i as f64]) + 1.0) / 2.0);
    }
    let mut result = Vec::new();
    for (i, strand) in strands.iter().enumerate() {
        let num = (noise[i] * (*strand as f64)) as usize;
        for _ in 0..num {
            result.push(LedColor::from_u32_rgb(0x0));
        }
        for _ in num..*strand {
            let rgb : [u8; 3] = color_rgb.into_format().into_raw();
            result.push(LedColor::from_u8_rgb(rgb));
        }
    }
    return result;
}

fn render_static(_t: f64, color: palette::Srgb, strands: &Vec<usize>) -> Vec<LedColor> {
    // let mut cumsum = Vec::new();
    // for strand in &strands {
    //     let prev = cumsum.last().copied().unwrap_or(0);
    //     cumsum.push(prev + strand);
    // }
    let on = LedColor::from_u8_rgb(color.into_format().into_raw());
    let mut result = Vec::new();
    for strand in strands {
        for _ in 0..*strand {
            result.push(on);
        }
    }
    return result;
}
