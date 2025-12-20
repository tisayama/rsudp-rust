use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Settings {
    /// UDP port to listen on
    #[arg(long, default_value_t = 8888)]
    pub port: u16,

    /// WebUI port to listen on
    #[arg(long, default_value_t = 8080)]
    pub web_port: u16,

    /// MiniSEED file(s) to process (simulation mode)
    #[arg(short, long)]
    pub file: Vec<String>,

    /// Run with mock data for WebUI stress testing

    #[arg(long, default_value_t = false)]
    pub mock: bool,

    /// Three channels for seismic intensity calculation (comma separated, e.g. ENE,ENN,ENZ)

    #[arg(long)]
    pub channels: Option<String>,
}
