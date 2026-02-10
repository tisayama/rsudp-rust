use crate::hue::config::HueConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use config::{Config, ConfigError, Environment, File};
use tracing::warn;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub struct Settings {
    #[serde(alias = "SETTINGS")]
    pub settings: SettingsSection,
    #[serde(alias = "PRINTDATA")]
    pub printdata: PrintDataSettings,
    #[serde(alias = "WRITE")]
    pub write: WriteSettings,
    #[serde(alias = "PLOT")]
    pub plot: PlotSettings,
    #[serde(alias = "FORWARD")]
    pub forward: ForwardSettings,
    #[serde(alias = "ALERT")]
    pub alert: AlertSettings,
    #[serde(alias = "ALERTSOUND")]
    pub alertsound: AlertSoundSettings,
    #[serde(alias = "CUSTOM")]
    pub custom: CustomSettings,
    #[serde(alias = "TWEETS")]
    pub tweets: TweetsSettings,
    #[serde(alias = "TELEGRAM")]
    pub telegram: TelegramSettings,
    #[serde(alias = "GOOGLECHAT")]
    pub googlechat: GoogleChatSettings,
    #[serde(alias = "DISCORD")]
    pub discord: DiscordSettings,
    #[serde(alias = "SNS")]
    pub sns: SnsSettings,
    #[serde(alias = "LINE")]
    pub line: LineSettings,
    #[serde(alias = "BLUESKY")]
    pub bluesky: BlueSkySettings,
    #[serde(alias = "RSAM")]
    pub rsam: RsamSettings,
    #[serde(alias = "HUE")]
    pub hue: HueConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct SettingsSection {
    #[serde(alias = "PORT")]
    pub port: u16,
    #[serde(alias = "STATION")]
    pub station: String,
    #[serde(alias = "OUTPUT_DIR")]
    pub output_dir: PathBuf,
    #[serde(alias = "DEBUG")]
    pub debug: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct PrintDataSettings {
    #[serde(alias = "ENABLED")]
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct WriteSettings {
    #[serde(alias = "ENABLED")]
    pub enabled: bool,
    #[serde(alias = "CHANNELS")]
    pub channels: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct PlotSettings {
    #[serde(alias = "ENABLED")]
    pub enabled: bool,
    #[serde(alias = "DURATION")]
    pub duration: u32,
    #[serde(alias = "REFRESH_INTERVAL")]
    pub refresh_interval: u32,
    #[serde(alias = "SPECTROGRAM")]
    pub spectrogram: bool,
    #[serde(alias = "FULLSCREEN")]
    pub fullscreen: bool,
    #[serde(alias = "KIOSK")]
    pub kiosk: bool,
    #[serde(alias = "EQ_SCREENSHOTS")]
    pub eq_screenshots: bool,
    #[serde(alias = "CHANNELS")]
    pub channels: Vec<String>,
    #[serde(alias = "FILTER_WAVEFORM")]
    pub filter_waveform: bool,
    #[serde(alias = "FILTER_SPECTROGRAM")]
    pub filter_spectrogram: bool,
    #[serde(alias = "FILTER_HIGHPASS")]
    pub filter_highpass: f64,
    #[serde(alias = "FILTER_LOWPASS")]
    pub filter_lowpass: f64,
    #[serde(alias = "FILTER_CORNERS")]
    pub filter_corners: u32,
    #[serde(alias = "SPECTROGRAM_FREQ_RANGE")]
    pub spectrogram_freq_range: bool,
    #[serde(alias = "UPPER_LIMIT")]
    pub upper_limit: f64,
    #[serde(alias = "LOWER_LIMIT")]
    pub lower_limit: f64,
    #[serde(alias = "LOGARITHMIC_Y_AXIS")]
    pub logarithmic_y_axis: bool,
    #[serde(alias = "DECONVOLVE")]
    pub deconvolve: bool,
    #[serde(alias = "UNITS")]
    pub units: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct ForwardSettings {
    #[serde(alias = "ENABLED")]
    pub enabled: bool,
    #[serde(alias = "ADDRESS")]
    pub address: Vec<String>,
    #[serde(alias = "PORT")]
    pub port: Vec<u16>,
    #[serde(alias = "CHANNELS")]
    pub channels: Vec<String>,
    #[serde(alias = "FWD_DATA")]
    pub fwd_data: bool,
    #[serde(alias = "FWD_ALARMS")]
    pub fwd_alarms: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct AlertSettings {
    #[serde(alias = "ENABLED")]
    pub enabled: bool,
    #[serde(alias = "CHANNEL")]
    pub channel: String,
    #[serde(alias = "STA")]
    pub sta: f64,
    #[serde(alias = "LTA")]
    pub lta: f64,
    #[serde(alias = "DURATION")]
    pub duration: f64,
    #[serde(alias = "THRESHOLD")]
    pub threshold: f64,
    #[serde(alias = "RESET")]
    pub reset: f64,
    #[serde(alias = "HIGHPASS")]
    pub highpass: f64,
    #[serde(alias = "LOWPASS")]
    pub lowpass: f64,
    #[serde(alias = "DECONVOLVE")]
    pub deconvolve: bool,
    #[serde(alias = "UNITS")]
    pub units: String,
    #[serde(alias = "ON_PLOT")]
    pub on_plot: bool,
    #[serde(alias = "ON_PLOT_END_LINE_COLOR")]
    pub on_plot_end_line_color: String,
    #[serde(alias = "ON_PLOT_START_LINE_COLOR")]
    pub on_plot_start_line_color: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct AlertSoundSettings {
    #[serde(alias = "ENABLED")]
    pub enabled: bool,
    #[serde(alias = "TRIGGER_FILE")]
    pub trigger_file: String,
    #[serde(alias = "DEFAULT_RESET_FILE")]
    pub default_reset_file: String,
    #[serde(alias = "INTENSITY_FILES")]
    pub intensity_files: std::collections::BTreeMap<String, String>,
}

impl Default for AlertSoundSettings {
    fn default() -> Self {
        // Empty map for default to avoid config crate parsing issues with keys like "5+" during default loading
        let intensity_files = std::collections::BTreeMap::new();
        
        Self {
            enabled: false,
            trigger_file: "".to_string(),
            default_reset_file: "".to_string(),
            intensity_files,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct CustomSettings {
    #[serde(alias = "ENABLED")]
    pub enabled: bool,
    #[serde(alias = "CODEFILE")]
    pub codefile: String,
    #[serde(alias = "WIN_OVERRIDE")]
    pub win_override: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct TweetsSettings {
    #[serde(alias = "ENABLED")]
    pub enabled: bool,
    #[serde(alias = "TWEET_IMAGES")]
    pub tweet_images: bool,
    #[serde(alias = "API_KEY")]
    pub api_key: String,
    #[serde(alias = "API_SECRET")]
    pub api_secret: String,
    #[serde(alias = "ACCESS_TOKEN")]
    pub access_token: String,
    #[serde(alias = "ACCESS_SECRET")]
    pub access_secret: String,
    #[serde(alias = "EXTRA_TEXT")]
    pub extra_text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct TelegramSettings {
    #[serde(alias = "ENABLED")]
    pub enabled: bool,
    #[serde(alias = "SEND_IMAGES")]
    pub send_images: bool,
    #[serde(alias = "TOKEN")]
    pub token: String,
    #[serde(alias = "CHAT_ID")]
    pub chat_id: String,
    #[serde(alias = "EXTRA_TEXT")]
    pub extra_text: String,
    #[serde(alias = "UPLOAD_TIMEOUT")]
    pub upload_timeout: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct GoogleChatSettings {
    #[serde(alias = "ENABLED")]
    pub enabled: bool,
    #[serde(alias = "WEBHOOK_URL")]
    pub webhook_url: String,
    #[serde(alias = "EXTRA_TEXT")]
    pub extra_text: String,
    #[serde(alias = "SEND_IMAGES")]
    pub send_images: bool,
    #[serde(alias = "S3_BUCKET_NAME")]
    pub s3_bucket_name: Option<String>,
    #[serde(alias = "S3_OBJECT_KEY_PREFIX")]
    pub s3_object_key_prefix: String,
    #[serde(alias = "S3_AWS_REGION")]
    pub s3_aws_region: Option<String>,
    #[serde(alias = "S3_UPLOAD_TIMEOUT_SECONDS")]
    pub s3_upload_timeout_seconds: u32,
    #[serde(alias = "AWS_ACCESS_KEY_ID")]
    pub aws_access_key_id: Option<String>,
    #[serde(alias = "AWS_SECRET_ACCESS_KEY")]
    pub aws_secret_access_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct DiscordSettings {
    #[serde(alias = "ENABLED")]
    pub enabled: bool,
    #[serde(alias = "WEBHOOK_URL")]
    pub webhook_url: String,
    #[serde(alias = "USE_EMBED")]
    pub use_embed: bool,
    #[serde(alias = "SEND_IMAGES")]
    pub send_images: bool,
    #[serde(alias = "EXTRA_TEXT")]
    pub extra_text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct SnsSettings {
    #[serde(alias = "ENABLED")]
    pub enabled: bool,
    #[serde(alias = "TOPIC_ARN")]
    pub topic_arn: String,
    #[serde(alias = "AWS_ACCESS_KEY_ID")]
    pub aws_access_key_id: Option<String>,
    #[serde(alias = "AWS_SECRET_ACCESS_KEY")]
    pub aws_secret_access_key: Option<String>,
    #[serde(alias = "AWS_REGION")]
    pub aws_region: String,
    #[serde(alias = "EXTRA_TEXT")]
    pub extra_text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct LineSettings {
    #[serde(alias = "ENABLED")]
    pub enabled: bool,
    #[serde(alias = "CHANNEL_ACCESS_TOKEN")]
    pub channel_access_token: String,
    #[serde(alias = "TO_IDS")]
    pub to_ids: String,
    #[serde(alias = "EXTRA_TEXT")]
    pub extra_text: String,
    #[serde(alias = "SEND_IMAGES")]
    pub send_images: bool,
    #[serde(alias = "S3_BUCKET_NAME")]
    pub s3_bucket_name: Option<String>,
    #[serde(alias = "S3_OBJECT_KEY_PREFIX")]
    pub s3_object_key_prefix: String,
    #[serde(alias = "S3_AWS_REGION")]
    pub s3_aws_region: Option<String>,
    #[serde(alias = "S3_UPLOAD_TIMEOUT_SECONDS")]
    pub s3_upload_timeout_seconds: u32,
    #[serde(alias = "AWS_ACCESS_KEY_ID")]
    pub aws_access_key_id: Option<String>,
    #[serde(alias = "AWS_SECRET_ACCESS_KEY")]
    pub aws_secret_access_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct BlueSkySettings {
    #[serde(alias = "ENABLED")]
    pub enabled: bool,
    #[serde(alias = "POST_IMAGES")]
    pub post_images: bool,
    #[serde(alias = "USERNAME")]
    pub username: String,
    #[serde(alias = "PASSWORD")]
    pub password: String,
    #[serde(alias = "EXTRA_TEXT")]
    pub extra_text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct RsamSettings {
    #[serde(alias = "ENABLED")]
    pub enabled: bool,
    #[serde(alias = "QUIET")]
    pub quiet: bool,
    #[serde(alias = "FWADDR")]
    pub fwaddr: String,
    #[serde(alias = "FWPORT")]
    pub fwport: u16,
    #[serde(alias = "FWFORMAT")]
    pub fwformat: String,
    #[serde(alias = "CHANNEL")]
    pub channel: String,
    #[serde(alias = "INTERVAL")]
    pub interval: u32,
    #[serde(alias = "DECONVOLVE")]
    pub deconvolve: bool,
    #[serde(alias = "UNITS")]
    pub units: String,
}

impl Default for SettingsSection {
    fn default() -> Self {
        Self {
            port: 8888,
            station: "Z0000".to_string(),
            output_dir: PathBuf::from("rsudp"),
            debug: true,
        }
    }
}

impl Default for PrintDataSettings {
    fn default() -> Self {
        Self { enabled: false }
    }
}

impl Default for WriteSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            channels: vec!["all".to_string()],
        }
    }
}

impl Default for PlotSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            duration: 90,
            refresh_interval: 0,
            spectrogram: true,
            fullscreen: false,
            kiosk: false,
            eq_screenshots: false,
            channels: vec!["all".to_string()],
            filter_waveform: false,
            filter_spectrogram: false,
            filter_highpass: 0.7,
            filter_lowpass: 2.0,
            filter_corners: 4,
            spectrogram_freq_range: false,
            upper_limit: 15.0,
            lower_limit: 0.0,
            logarithmic_y_axis: false,
            deconvolve: true,
            units: "CHAN".to_string(),
        }
    }
}

impl Default for ForwardSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            address: vec!["192.168.1.254".to_string()],
            port: vec![8888],
            channels: vec!["all".to_string()],
            fwd_data: true,
            fwd_alarms: false,
        }
    }
}

impl Default for AlertSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            channel: "HZ".to_string(),
            sta: 6.0,
            lta: 30.0,
            duration: 0.0,
            threshold: 1.1,
            reset: 0.5,
            highpass: 0.1,
            lowpass: 2.0,
            deconvolve: false,
            units: "VEL".to_string(),
            on_plot: false,
            on_plot_end_line_color: "#D72638".to_string(),
            on_plot_start_line_color: "#4C8BF5".to_string(),
        }
    }
}

impl Default for CustomSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            codefile: "n/a".to_string(),
            win_override: false,
        }
    }
}

impl Default for TweetsSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            tweet_images: true,
            api_key: "n/a".to_string(),
            api_secret: "n/a".to_string(),
            access_token: "n/a".to_string(),
            access_secret: "n/a".to_string(),
            extra_text: "".to_string(),
        }
    }
}

impl Default for TelegramSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            send_images: true,
            token: "n/a".to_string(),
            chat_id: "n/a".to_string(),
            extra_text: "".to_string(),
            upload_timeout: 10,
        }
    }
}

impl Default for GoogleChatSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            webhook_url: "n/a".to_string(),
            extra_text: "".to_string(),
            send_images: false,
            s3_bucket_name: None,
            s3_object_key_prefix: "rsudp/googlechat/".to_string(),
            s3_aws_region: None,
            s3_upload_timeout_seconds: 3,
            aws_access_key_id: None,
            aws_secret_access_key: None,
        }
    }
}

impl Default for DiscordSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            webhook_url: "n/a".to_string(),
            use_embed: true,
            send_images: true,
            extra_text: "".to_string(),
        }
    }
}

impl Default for SnsSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            topic_arn: "n/a".to_string(),
            aws_access_key_id: None,
            aws_secret_access_key: None,
            aws_region: "n/a".to_string(),
            extra_text: "".to_string(),
        }
    }
}

impl Default for LineSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            channel_access_token: "n/a".to_string(),
            to_ids: "".to_string(),
            extra_text: "".to_string(),
            send_images: true,
            s3_bucket_name: None,
            s3_object_key_prefix: "rsudp/line/".to_string(),
            s3_aws_region: None,
            s3_upload_timeout_seconds: 3,
            aws_access_key_id: None,
            aws_secret_access_key: None,
        }
    }
}

impl Default for BlueSkySettings {
    fn default() -> Self {
        Self {
            enabled: false,
            post_images: true,
            username: "n/a".to_string(),
            password: "n/a".to_string(),
            extra_text: "".to_string(),
        }
    }
}

impl Default for RsamSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            quiet: true,
            fwaddr: "192.168.1.254".to_string(),
            fwport: 8887,
            fwformat: "LITE".to_string(),
            channel: "HZ".to_string(),
            interval: 10,
            deconvolve: false,
            units: "VEL".to_string(),
        }
    }
}

impl Settings {
    pub fn new(config_path: Option<PathBuf>) -> Result<Self, ConfigError> {
        let mut builder = Config::builder();

        // 1. Load defaults
        let default_settings = Settings::default();
        builder = builder.add_source(config::Config::try_from(&default_settings)?);

        // 2. Load from file if specified
        if let Some(path) = config_path {
            if path.exists() {
                builder = builder.add_source(File::from(path));
            } else {
                warn!("Configuration file not found: {:?}", path);
            }
        } else {
            // Standard search path
            if let Some(home) = dirs::home_dir() {
                let toml_path = home.join(".rsudp").join("settings.toml");
                let yaml_path = home.join(".rsudp").join("settings.yaml");

                if toml_path.exists() {
                    builder = builder.add_source(File::from(toml_path));
                } else if yaml_path.exists() {
                    builder = builder.add_source(File::from(yaml_path));
                }
            }
        }

        // 3. Environment variables
        builder = builder.add_source(
            Environment::with_prefix("RUSTRSUDP")
                .separator("__")
                .try_parsing(true)
        );

        let config = builder.build()?;
        
        // T009: Detect unknown fields
        if let Ok(table) = config.clone().try_deserialize::<serde_json::Value>() {
            if let Some(map) = table.as_object() {
                let known_sections = ["settings", "printdata", "write", "plot", "forward", "alert", "alertsound", "custom", "tweets", "telegram", "googlechat", "discord", "sns", "line", "bluesky", "rsam", "hue"];
                for key in map.keys() {
                    let lower_key = key.to_lowercase();
                    if !known_sections.contains(&lower_key.as_str()) {
                        warn!("Unknown configuration section: {}", key);
                    }
                }
            }
        }

        config.try_deserialize()
    }

    pub fn dump(&self, format: &str) -> Result<String, Box<dyn std::error::Error>> {
        match format.to_lowercase().as_str() {
            "toml" => Ok(toml::to_string_pretty(self)?),
            "yaml" | "yml" => Ok(serde_yaml::to_string(self)?),
            _ => Err("Unsupported format".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File as StdFile;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.settings.port, 8888);
        assert_eq!(settings.settings.station, "Z0000");
    }

    #[test]
    fn test_load_toml() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("settings.toml");
        let mut file = StdFile::create(&config_path).unwrap();
        writeln!(file, "[settings]\nport = 1234\nstation = \"TEST1\"").unwrap();

        let settings = Settings::new(Some(config_path)).unwrap();
        assert_eq!(settings.settings.port, 1234);
        assert_eq!(settings.settings.station, "TEST1");
        assert_eq!(settings.plot.duration, 90);
    }

    #[test]
    fn test_load_yaml() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("settings.yaml");
        let mut file = StdFile::create(&config_path).unwrap();
        writeln!(file, "settings:\n  port: 4321\n  station: \"YAML1\"").unwrap();

        let settings = Settings::new(Some(config_path)).unwrap();
        assert_eq!(settings.settings.port, 4321);
        assert_eq!(settings.settings.station, "YAML1");
    }

    #[test]
    fn test_dump_toml() {
        let settings = Settings::default();
        let dumped = settings.dump("toml").unwrap();
        assert!(dumped.contains("port = 8888"));
        assert!(dumped.contains("station = \"Z0000\""));
    }
}
