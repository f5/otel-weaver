// SPDX-License-Identifier: Apache-2.0

//! Manage command line arguments

use crate::gen_client_sdk::GenClientSdkParams;
use crate::languages::LanguagesParams;
use crate::resolve::ResolveParams;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Command line arguments.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    /// List of supported commands
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Supported commands.
#[derive(Subcommand)]
pub enum Commands {
    /// Resolve a schema file and print the result
    Resolve(ResolveParams),
    /// Generate a client SDK (application)
    GenClientSdk(GenClientSdkParams),
    /// Generate a client API (third party)
    GenClientApi {
        /// Schema file used to generate the client API
        #[arg(short, long, value_name = "FILE")]
        schema: PathBuf,
    },
    /// List of supported languages
    Languages(LanguagesParams),
}
