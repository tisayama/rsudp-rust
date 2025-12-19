use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Settings {
    /// UDP port to listen on
    #[arg(long, default_value_t = 8888)]
    pub port: u16,

        /// MiniSEED file(s) to process (simulation mode)

        #[arg(short, long)]

        pub file: Vec<String>,

    

        /// Run with mock data for WebUI stress testing

        #[arg(long, default_value_t = false)]

        pub mock: bool,

    }

    