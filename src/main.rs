mod rpc;
mod replay;

use clap::{Command, Arg};
use std::sync::Mutex;
use lazy_static::lazy_static;

use crate::rpc::format::hex_to_decimal;
use crate::rpc::format::format_number_input;
use rpc::rpc::RpcConnection;
use crate::replay::*;

// Settings flags

#[derive(Default)]
pub struct AppConfig {
    pub exit_on_tx_fail: bool,
}

lazy_static! {
    static ref APP_CONFIG: Mutex<AppConfig> = Mutex::new(AppConfig::default());
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("sothis")
        .version("0.1.2")
        .author("makemake <vukasin@gostovic.me>")
        .about("Tool for replaying historical transactions. Designed to be used with anvil")
        .arg(Arg::new("source_rpc")
            .long("source_rpc")
            .short('s')
            .num_args(1..)
            .required(true)
            .help("HTTP JSON-RPC of the node we're querying data from"))
        .arg(Arg::new("terminal_block")
            .long("terminal_block")
            .short('b')
            .num_args(1..)
            .help("Block we're replaying until"))
        .arg(Arg::new("replay_rpc")
            .long("replay_rpc")
            .short('r')
            .num_args(1..)
            .required(true)
            .help("HTTP JSON-RPC of the node we're replaying data to"))
        .arg(Arg::new("mode")
            .long("mode")
            .short('m')
            .num_args(1..)
            .default_value("historic")
            .help("Choose between live replay or historic"))
        .arg(Arg::new("replay_node_type")
            .long("replay_node_type")
            .short('t')
            .num_args(1..)
            .default_value("anvil")
            .help("Choose between hardhat or anvil"))
        .arg(Arg::new("exit_on_tx_fail")
            .long("exit_on_tx_fail")
            .num_args(0..)
            .help("Exit the program if a transaction fails"))
        .get_matches();

    let source_rpc: String = matches.get_one::<String>("source_rpc").expect("required").to_string();
    let replay_rpc: String = matches.get_one::<String>("replay_rpc").expect("required").to_string();
    let mode: String = matches.get_one::<String>("mode").expect("required").to_string();

    let mut app_config = APP_CONFIG.lock().unwrap();
    app_config.exit_on_tx_fail = matches.get_occurrences::<String>("exit_on_tx_fail").is_some();

    let source_rpc = RpcConnection::new(source_rpc);
    let replay_rpc = RpcConnection::new(replay_rpc);
    
    match mode.as_str() {
        "historic" => {
            println!("Replaying in historic mode...");
            
            let block: String = matches.get_one::<String>("terminal_block").expect("required").to_string();
            let block = format_number_input(&block);

            replay_historic_blocks(source_rpc, replay_rpc, hex_to_decimal(&block)?).await?;
        },
        "live" => {
            println!("Replaying live blocks...");
            replay_live(replay_rpc, source_rpc).await?;
        }
        &_ => {
            // handle this properly later
            panic!("Mode does not exist!");
        },
    }
    
    Ok(())
}
