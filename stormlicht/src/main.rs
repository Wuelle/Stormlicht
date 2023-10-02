#![feature(panic_update_hook)]

mod browser_application;

use std::process::ExitCode;

use browser_application::BrowserApplication;

use cli::CommandLineArgumentParser;

#[derive(Debug, Default, CommandLineArgumentParser)]
struct ArgumentParser {
    #[argument(
        may_be_omitted,
        positional,
        short_name = 'u',
        long_name = "URL",
        description = "URL to load"
    )]
    url: Option<String>,

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

pub fn main() -> ExitCode {
    // Register a custom panic handler
    std::panic::update_hook(move |prev, info| {
        eprintln!(
            "The browser has panicked. This is a bug. Please open an issue at {}, including the debug information below. Thanks!\n", 
            env!("CARGO_PKG_REPOSITORY")
        );
        prev(info);
    });

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    #[cfg(target_os = "linux")]
    if unsafe { geteuid() } == 0 {
        log::error!("Refusing to run as root");
        return ExitCode::FAILURE;
    }

    let arguments = match ArgumentParser::parse() {
        Ok(arguments) => arguments,
        Err(_) => {
            println!("{}", ArgumentParser::help());
            return ExitCode::FAILURE;
        },
    };

    if arguments.help {
        println!("{}", ArgumentParser::help());
        return ExitCode::SUCCESS;
    }

    if arguments.version {
        println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

    BrowserApplication::run(arguments.url.as_deref())
}
