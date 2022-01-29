use clap::Parser;

/// Provides a control interface for WS2811 LED Light strips.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct DaemonArgs {
    /// Path to the listening socket of the daemon.
    #[clap(short, long)]
    pub unix_socket: String,

	/// The PWM channel to which the LED strip is connected. Usually 0 or 1.
    #[clap(short, long)]
    pub channel: usize,

	/// The DMA offset number.
    #[clap(short, long)]
    pub dma: i32,

	/// The pin to which the LED strip is attached
    #[clap(short, long)]
    pub pin: i32,

    /// How many LEDs the strip contains.
    #[clap(short, long)]
    pub leds_count: usize,
}
