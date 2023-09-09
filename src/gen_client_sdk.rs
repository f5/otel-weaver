// SPDX-License-Identifier: Apache-2.0

//! Generate a client API (third party)

use std::path::PathBuf;
use clap::Parser;
use logger::Logger;
use template::GeneratorConfig;
use template::sdkgen::ClientSdkGenerator;

/// Parameters for the `gen-client-sdk` command
#[derive(Parser)]
pub struct GenClientSdkParams {
    /// Schema file to resolve
    #[arg(short, long, value_name = "FILE")]
    schema: PathBuf,

    /// Language to generate the client SDK for
    #[arg(short, long)]
    language: String,
}

/// Generate a client SDK (application)
pub fn command_gen_client_sdk(log: &mut Logger, params: &GenClientSdkParams) {
    log.loading(&format!("Generating client SDK for language {}", params.language));
    if let Err(e) = ClientSdkGenerator::try_new(&params.language, GeneratorConfig::default()) {
        log.error(&format!("{}", e));
        return;
    }
    log.success("Generated client SDK");
}
