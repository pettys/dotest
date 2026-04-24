use anyhow::Result;
use crate::core::executor::discover_tests;
use crate::core::tree::build_flat_tree;

pub(crate) mod config;
mod discovery_cache;
mod filter;
mod interactive;
mod layout;
mod manual_watch;
mod output;

pub fn run() -> Result<()> {
    let config = config::RunConfig::load();

    let tests = if let Some(cached) = discovery_cache::try_load_cached_tests() {
        cached
    } else {
        println!("Discovering tests (this may take a moment)...");
        let tests = discover_tests(true, config.no_restore)?;
        discovery_cache::save_discovery_cache(&tests)?;
        tests
    };

    if tests.is_empty() {
        println!("No tests found.");
        return Ok(());
    }
    let mut tree = build_flat_tree(&tests);
    interactive::run_interactive_loop(&mut tree, config)
}
