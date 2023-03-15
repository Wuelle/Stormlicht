#![feature(panic_update_hook)]

mod browser_application;

use anyhow::{anyhow, Result};
use browser_application::BrowserApplication;

use cli::CommandLineArgumentParser;
use widgets::application::Application;

#[derive(Debug, Default, CommandLineArgumentParser)]
struct ArgumentParser {
    #[argument(optional, positional, long_name = "URL")]
    _url: Option<String>,

    #[argument(
        flag,
        short_name = 'h',
        long_name = "help",
        description = "Show this help menu"
    )]
    help: bool,

    #[argument(
        flag,
        short_name = 'v',
        long_name = "version",
        description = "Show browser version"
    )]
    version: bool,
}

#[cfg(target_os = "linux")]
#[link(name = "c")]
extern "C" {
    fn geteuid() -> u32;
}

pub fn main() -> Result<()> {
    // Register a custom panic handler
    std::panic::update_hook(move |prev, info| {
        println!("The browser has panicked. This is a bug. Please open an issue at {}, including the debug information below. Thanks!\n", env!("CARGO_PKG_REPOSITORY"));
        prev(info);
    });

    #[cfg(target_os = "linux")]
    if unsafe { geteuid() } == 0 {
        return Err(anyhow!("Refusing to run as root"));
    }

    let arguments = match ArgumentParser::parse() {
        Ok(arguments) => arguments,
        Err(_) => {
            println!("{}", ArgumentParser::help());
            return Ok(());
        },
    };

    if arguments.help {
        println!("{}", ArgumentParser::help());
        return Ok(());
    }

    if arguments.version {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let mut application = BrowserApplication::default();
    application.run()
}