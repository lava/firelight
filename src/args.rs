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

/// Starts a REST Api and web interface to control
/// firelight via homeassistant or a browser.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct ServerArgs {
    /// Path to the listening socket of the daemon.
    #[clap(short, long)]
    pub daemon_socket: String,

    /// The listen address to bind to.
    #[clap(short, long, default_value = "localhost:1313")]
    pub bind: String,

    /// A unique identifier for this server instance.
    #[clap(short, long, default_value="firelight-lamp")]
    pub instance_name: String,

    /// The logical arrangement of the strip into vertical strands.
    #[clap(short, long, multiple_occurrences = false, multiple_values = true, use_delimiter= true)]
    pub strands: Vec<usize>,
}

