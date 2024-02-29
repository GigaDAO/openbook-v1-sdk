/*
COIN == BASE
PC == QUOTE
 */
mod load_oo_state;
mod initialize_oo_account;
mod utils;
mod ob_client;

use std::num::NonZeroU64;
use std::ops::DerefMut;
use std::str::FromStr;
use debug_ignore::DebugIgnore;
use openbook_dex::critbit::{SlabView};
use openbook_dex::instruction::SelfTradeBehavior;
use openbook_dex::matching::{OrderType, Side};
use openbook_dex::state::{MarketState, ToAlignedBytes};
use solana_program::{
    pubkey::Pubkey,
};
use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use rand::random;
use solana_rpc_client_api::config::RpcSendTransactionConfig;
use crate::load_oo_state::{load_oo_state, OpenOrderState};
use crate::ob_client::load_ob_client;
use crate::utils::{create_account_info_from_account, get_unix_secs, read_keypair};

const SOL_USDC_MARKET_ID: &str = "8BnEgHoWFysVcuFFX7QztDmzuH8r5ZFvyP3sYwn1XTh6";
const OPENBOOK_V1_PROGRAM_ID: &str = "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX";

#[derive(Debug)]
pub struct ObClient {
    pub rpc_client: DebugIgnore<RpcClient>,
    pub market_account: Pubkey,
    pub open_orders_account: Pubkey,
    pub request_queue: Pubkey,
    pub event_queue: Pubkey,
    pub base_ata: Pubkey,
    pub quote_ata: Pubkey,
    pub keypair: Keypair,
    pub coin_vault: Pubkey, // base
    pub pc_vault: Pubkey, //quote
    pub program_id: Pubkey,
    pub vault_signer_key: Pubkey,
    pub oo_state: OpenOrderState,
}


pub fn load_market() -> anyhow::Result<()>{

    let ob_client = load_ob_client()?;
    println!("{:?}", ob_client);

    // let target_size_usdc = 1.0;
    // let target_usdc_lots_w_fee = (target_size_usdc * 1e6 * 1.1) as u64;
    // let price = oo_state.max_bid as f64 / 1e3 - 30.;
    // let new_bid = (price * 1e3) as u64;
    // let target_amount_wsol = target_size_usdc / price;
    // let target_wsol_lots = (target_amount_wsol * 1e3) as u64;
    //
    // let limit_price = NonZeroU64::new(new_bid).unwrap();
    // let max_coin_qty = NonZeroU64::new(target_wsol_lots).unwrap(); // max wsol lots
    // let max_native_pc_qty_including_fees = NonZeroU64::new(target_usdc_lots_w_fee).unwrap(); // max usdc lots + fees
    //
    // //
    //
    //
    // // TODO extract
    // if place_limit_bid {
    //     let place_order_ix = openbook_dex::instruction::new_order(
    //         &market_account_binding,
    //         &orders_key,
    //         &request_queue,
    //         &event_queue,
    //         &oo_state.bids_address,
    //         &oo_state.asks_address,
    //         &usdc_ata,
    //         &keypair.pubkey(),
    //         &coin_vault,
    //         &pc_vault,
    //         &anchor_spl::token::ID,
    //         &solana_program::sysvar::rent::ID,
    //         None,
    //         &program_id_binding,
    //         Side::Bid,
    //         limit_price,
    //         max_coin_qty,
    //         OrderType::PostOnly,
    //         random::<u64>(),
    //         SelfTradeBehavior::AbortTransaction,
    //         u16::MAX,
    //         max_native_pc_qty_including_fees,
    //         (get_unix_secs() + 30) as i64,
    //     )?;
    //
    //     // place order
    //     let mut instructions = Vec::new();
    //     instructions.push(place_order_ix);
    //
    //     let recent_hash = rpc_client.get_latest_blockhash()?;
    //     let txn = Transaction::new_signed_with_payer(
    //         &instructions,
    //         Some(&keypair.pubkey()),
    //         &[&keypair],
    //         recent_hash,
    //     );
    //
    //     let mut config = RpcSendTransactionConfig::default();
    //     config.skip_preflight = true;
    //     let r = rpc_client.send_transaction_with_config(&txn, config);
    //     println!("got results: {:?}", r);
    // }
    //
    // // TODO extract
    // if cancel_order {
    //     let ix = openbook_dex::instruction::cancel_order(
    //         &program_id_binding,
    //         &market_account_binding,
    //         &oo_state.bids_address,
    //         &oo_state.asks_address,
    //         &orders_key,
    //         &keypair.pubkey(),
    //         &event_queue,
    //         Side::Bid,
    //         order_id_0,
    //     )?;
    //     // place order
    //     let mut instructions = Vec::new();
    //     instructions.push(ix);
    //
    //     let recent_hash = rpc_client.get_latest_blockhash()?;
    //     let txn = Transaction::new_signed_with_payer(
    //         &instructions,
    //         Some(&keypair.pubkey()),
    //         &[&keypair],
    //         recent_hash,
    //     );
    //
    //     let mut config = RpcSendTransactionConfig::default();
    //     config.skip_preflight = true;
    //     let r = rpc_client.send_transaction_with_config(&txn, config);
    //     println!("got results: {:?}", r);
    // }
    //
    // // TODO extract
    // if settle_balance {
    //
    //     let ix = openbook_dex::instruction::settle_funds(
    //         &program_id_binding,
    //         &market_account_binding,
    //         &anchor_spl::token::ID,
    //         &orders_key,
    //         &keypair.pubkey(),
    //         &coin_vault,
    //         &wsol_ata,
    //         &pc_vault,
    //         &usdc_ata,
    //         None,
    //         &vault_signer_key,
    //     )?;
    //
    //     // place order
    //     let mut instructions = Vec::new();
    //     instructions.push(ix);
    //
    //     let recent_hash = rpc_client.get_latest_blockhash()?;
    //     let txn = Transaction::new_signed_with_payer(
    //         &instructions,
    //         Some(&keypair.pubkey()),
    //         &[&keypair],
    //         recent_hash,
    //     );
    //
    //     let mut config = RpcSendTransactionConfig::default();
    //     config.skip_preflight = true;
    //     let r = rpc_client.send_transaction_with_config(&txn, config);
    //     println!("got results: {:?}", r);
    //
    // }


    Ok(())

}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        load_market().unwrap();
    }
}
