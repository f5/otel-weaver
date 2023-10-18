use clap::Parser;

use weaver_logger::Logger;

use crate::cli::{Cli, Commands};
use crate::gen_client_api::command_gen_client_api;
use crate::gen_client_sdk::command_gen_client_sdk;
use crate::resolve::command_resolve;

mod cli;
mod gen_client_api;
mod gen_client_sdk;
mod languages;
mod resolve;
mod search;

fn main() {
    let cli = Cli::parse();
    let log = Logger::new(cli.debug);

    match &cli.command {
        Some(Commands::Resolve(params)) => {
            command_resolve(log, params);
        }
        Some(Commands::GenClientSdk(params)) => {
            command_gen_client_sdk(log, params);
        }
        Some(Commands::GenClientApi { schema }) => {
            command_gen_client_api(log, schema);
        }
        Some(Commands::Languages(params)) => {
            languages::command_languages(log, params);
        }
        Some(Commands::Search(params)) => {
            search::command_search(log, params);
        }
        None => {}
    }
}
