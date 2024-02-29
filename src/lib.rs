/*
COIN == BASE
PC == QUOTE
 */
mod load_oo_state;
mod initialize_oo_account;
mod utils;
mod ob_client;
mod ixs;

use std::num::NonZeroU64;
use std::ops::DerefMut;
use std::str::FromStr;
use debug_ignore::DebugIgnore;
use openbook_dex::critbit::{SlabView};
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
use crate::ixs::cancel_limit_order::cancel_all_limit_orders;
use crate::ixs::place_limit_order::place_limit_order;
use crate::ixs::settle_balance::settle_balance;
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
    pub claimable: bool,
}


pub fn test_place_and_cancel() -> anyhow::Result<()>{

    let mut ob_client = load_ob_client()?;

    if ob_client.claimable {
        settle_balance(&mut ob_client)?;
    }

    // place_limit_order(&mut ob_client, 1., Side::Bid, 20.)?;
    // place_limit_order(&mut ob_client, 1., Side::Bid, 30.)?;
    // place_limit_order(&mut ob_client, 1., Side::Ask, 20.)?;
    // place_limit_order(&mut ob_client, 1., Side::Ask, 30.)?;

    cancel_all_limit_orders(&mut ob_client)?;

    Ok(())
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        test_place_and_cancel().unwrap();
    }
}
