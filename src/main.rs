// Copyright 2019 PolkaWorld.

extern crate futures;
extern crate jsonrpc_client_core;
extern crate jsonrpc_core;
extern crate jsonrpc_ws_server;
extern crate parking_lot;
extern crate serde;
extern crate url;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;
extern crate hex;
extern crate node_template_runtime as runtime;
extern crate parity_codec as codec;
extern crate sr_primitives;
extern crate srml_support;
extern crate srml_system;
extern crate substrate_primitives;

mod substrate_rpc;
mod ws;

use self::ws::{Rpc, RpcError};
use clap::load_yaml;
use jsonrpc_core::Notification;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc;

pub fn read_a_file() -> std::io::Result<Vec<u8>> {
    let mut file = try!(File::open("upgrade.wasm"));

    let mut data = Vec::new();
    try!(file.read_to_end(&mut data));

    return Ok(data);
}

fn substrate_thread(
    send_tx: mpsc::Sender<jsonrpc_ws_server::ws::Message>,
) -> Result<Rpc, RpcError> {
    let port = 8087;
    Rpc::new(&format!("ws://127.0.0.1:{}", port), send_tx)
}

fn print_usage(matches: &clap::ArgMatches) {
    println!("{}", matches.usage());
}

fn execute(matches: clap::ArgMatches) {
    let (send_tx, recv_tx) = mpsc::channel();
    let mut substrate_client = substrate_thread(send_tx.clone()).unwrap();
    let substrate_genesis_hash = substrate_rpc::genesis_hash(&mut substrate_client);
    println!("substrate genesis hash: {:?}", substrate_genesis_hash);
    let raw_seed = substrate_rpc::RawSeed::new("Alice");
    let account = raw_seed.account_id();
    let index = substrate_rpc::account_nonce(&mut substrate_client, &account);

    match matches.subcommand() {
        ("transfer", Some(matches)) => {
            /*let to = matches.value_of("to")
                .expect("parameter is required; thus it can't be None; qed");
            let amount = matches.value_of("amount")
                .expect("parameter is required; thus it can't be None; qed");*/
            let tx = substrate_rpc::generate_transfer_tx(
                &raw_seed,
                account,
                index,
                substrate_genesis_hash,
            );
            substrate_rpc::transfer(&mut substrate_client, tx);
        }
        _ => print_usage(&matches),
    }

    let msg = recv_tx.recv().unwrap();
    let msg = msg.into_text().unwrap();
    let des: Notification = serde_json::from_str(&msg).unwrap();
    let des: serde_json::Map<String, serde_json::Value> = des.params.parse().unwrap();
    let sub_id = &des["subscription"];
    println!(
        "----subscribe extrinsic return sub_id:{:?}----result:{:?}---",
        sub_id, des["result"]
    );
}
fn main() {
    let _ = env_logger::try_init();
    let yaml = load_yaml!("cli.yml");
    let matches = clap::App::from_yaml(yaml)
        .version(env!("CARGO_PKG_VERSION"))
        .get_matches();
    execute(matches);
}
