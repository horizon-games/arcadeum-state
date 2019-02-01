//! Substrate Node Template CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

extern crate futures;
#[macro_use]
extern crate error_chain;
extern crate tokio;
#[macro_use]
extern crate log;
extern crate substrate_cli;
extern crate substrate_client as client;
extern crate substrate_consensus_aura as consensus;
extern crate substrate_primitives as primitives;
#[macro_use]
extern crate substrate_network as network;
#[macro_use]
extern crate substrate_executor;
extern crate substrate_basic_authorship as basic_authorship;
extern crate substrate_transaction_pool as transaction_pool;
#[macro_use]
extern crate substrate_service;
extern crate node_executor;
extern crate rust_substrate_prototype_runtime;
extern crate substrate_inherents as inherents;

mod chain_spec;
mod cli;
mod service;

pub use substrate_cli::{error, IntoExit, VersionInfo};

fn run() -> cli::error::Result<()> {
    let version = VersionInfo {
        name: "Substrate Node",
        commit: env!("VERGEN_SHA_SHORT"),
        version: env!("CARGO_PKG_VERSION"),
        executable_name: "rust-substrate-prototype",
        author: "William Hua",
        description: "rust-substrate-prototype",
    };
    cli::run(::std::env::args(), cli::Exit, version)
}

quick_main!(run);
