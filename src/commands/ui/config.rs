use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub(super) enum Verbosity {
    Minimal,
    Normal,
    Detailed,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub(super) enum OutputMode {
    Split,
    Fullscreen,
}

fn default_output_mode() -> OutputMode {
    OutputMode::Split
}

fn default_manual_watch_delay_ms() -> u32 {
    2000
}

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct RunConfig {
    pub no_build: bool,
    pub verbosity: Verbosity,
    pub cache_tests: bool,
    #[serde(default = "default_output_mode")]
    pub output_mode: OutputMode,
    /// When set, a background watcher re-runs **only the tests you have checked in the tree**
    /// when `.cs` files change (debounced). For this option, you choose the scope.
	/// In the future, maybe add an automatic scope based on impact analysis.
	/// I tried, but it didn't work well. Halting for now.
    #[serde(default)]
    pub manual_watch_enabled: bool,
    #[serde(default = "default_manual_watch_delay_ms")]
    pub manual_watch_delay_ms: u32,
}

impl Default for RunConfig {
    fn default() -> Self {
        RunConfig {
            no_build: true,
            verbosity: Verbosity::Normal,
            cache_tests: false,
            output_mode: OutputMode::Split,
            manual_watch_enabled: false,
            manual_watch_delay_ms: 2000,
        }
    }
}

impl RunConfig {
    pub(super) fn load() -> Self {
        if let Ok(s) = std::fs::read_to_string(".dotest.yml") {
            if let Ok(cfg) = serde_yaml::from_str(&s) {
                return cfg;
            }
        }
        RunConfig::default()
    }

    pub(super) fn save(&self) {
        if let Ok(s) = serde_yaml::to_string(self) {
            let _ = std::fs::write(".dotest.yml", s);
        }
    }
}
