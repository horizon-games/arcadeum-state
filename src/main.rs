//! Substrate Node Template CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
#[macro_use]
extern crate substrate_network as network;
#[macro_use]
extern crate substrate_executor;
#[macro_use]
extern crate substrate_service;

mod chain_spec;
mod cli;
mod service;

pub use substrate_cli::{error, IntoExit, VersionInfo};

fn run() -> cli::error::Result<()> {
    let version = VersionInfo {
        name: "Substrate Node",
        commit: env!("VERGEN_SHA_SHORT"),
        version: env!("CARGO_PKG_VERSION"),
        executable_name: "arcadeum-chain",
        author: "William Hua",
        description: "arcadeum-chain",
        support_url: "support.anonymous.an",
    };
    cli::run(::std::env::args(), cli::Exit, version)
}

quick_main!(run);
