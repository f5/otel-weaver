use clap::Parser;

use logger::Logger;

use crate::cli::{Cli, Commands};
use crate::gen_client_api::command_gen_client_api;
use crate::gen_client_sdk::command_gen_client_sdk;
use crate::resolve::command_resolve;

mod cli;
mod gen_client_api;
mod gen_client_sdk;
mod languages;
mod resolve;

fn main() {
    let cli = Cli::parse();
    let mut log = Logger::new(cli.debug);

    match &cli.command {
        Some(Commands::Resolve(params)) => {
            command_resolve(&mut log, params);
        }
        Some(Commands::GenClientSdk(params)) => {
            command_gen_client_sdk(&mut log, params);
        }
        Some(Commands::GenClientApi { schema }) => {
            command_gen_client_api(&mut log, schema);
        }
        Some(Commands::Languages(params)) => {
            languages::command_languages(&mut log, params);
        }
        None => {}
    }
}
