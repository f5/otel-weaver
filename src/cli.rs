// SPDX-License-Identifier: Apache-2.0

//! Manage command line arguments

use std::path::PathBuf;
use clap::{Parser, Subcommand};
use crate::resolve::Resolve;

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
    Resolve(Resolve),
    /// Generate a client SDK (application)
    GenClientSdk {
        /// Schema file used to generate the client SDK
        #[arg(short, long, value_name = "FILE")]
        schema: PathBuf,
    },
    /// Generate a client API (third party)
    GenClientApi {
        /// Schema file used to generate the client API
        #[arg(short, long, value_name = "FILE")]
        schema: PathBuf,
    },
}