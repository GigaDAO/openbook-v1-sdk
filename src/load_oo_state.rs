use std::cell::RefMut;
use openbook_dex::state::MarketState;
use solana_program::pubkey::Pubkey;
use solana_rpc_client::rpc_client::RpcClient;
use crate::{create_account_info_from_account, OPENBOOK_V1_PROGRAM_ID};
use crate::utils::get_bids_asks_addresses;

#[derive(Debug)]
pub struct OpenOrderState {
    pub min_ask: u64,
    pub max_bid: u64,
    pub open_asks: Vec<u128>,
    pub open_bids: Vec<u128>,
    pub bids_address: Pubkey,
    pub asks_address: Pubkey,
}

pub fn load_oo_state(rpc_client: &mut RpcClient, market_state: RefMut<MarketState>, orders_key: &Pubkey) -> anyhow::Result<OpenOrderState>{

    let bids_address;
    let asks_address;

    let mut open_bids = Vec::new();
    let mut open_asks = Vec::new();

    let mut max_bid = 0;
    let mut min_ask = 0;

    let program_id_binding = OPENBOOK_V1_PROGRAM_ID.parse().unwrap();
    (bids_address, asks_address) = get_bids_asks_addresses(&market_state);

    // load bids
    {
        let mut bids_account = rpc_client.get_account(&bids_address)?;
        let mut bids_info = create_account_info_from_account(&mut bids_account, &bids_address, &program_id_binding, false, false);
        let mut bids = market_state.load_bids_mut(&mut bids_info)?;
        loop {
            let node = bids.remove_max();
            match node {
                Some(node) => {
                    // TODO extract all instance of this conversion... use a From trait or just a util
                    let owner_byes = node.owner();
                    let mut bytes: [u8; 32] = [0; 32];
                    for i in 0..4 {
                        bytes[i * 8..i * 8 + 8].copy_from_slice(&owner_byes[i].to_le_bytes());
                    }
                    let owner_address = Pubkey::from(bytes);
                    //
                    let order_id = node.order_id();
                    let price_raw = node.price().get();
                    if max_bid == 0 {
                        max_bid = price_raw;
                    }
                    if &owner_address == orders_key {
                        open_bids.push(order_id);
                    }
                }
                None => {
                    break;
                }
            }
        }
    }

    // load asks
    {
        let mut asks_accounts = rpc_client.get_account(&asks_address)?;
        let mut asks_info = create_account_info_from_account(&mut asks_accounts, &asks_address, &program_id_binding, false, false);
        let mut asks = market_state.load_asks_mut(&mut asks_info)?;
        loop {
            let node = asks.remove_min();
            match node {
                Some(node) => {
                    let owner_byes = node.owner();
                    let mut bytes: [u8; 32] = [0; 32];
                    for i in 0..4 {
                        bytes[i * 8..i * 8 + 8].copy_from_slice(&owner_byes[i].to_le_bytes());
                    }
                    let owner_address = Pubkey::from(bytes);
                    //
                    let order_id = node.order_id();
                    let price_raw = node.price().get();
                    if min_ask == 0 {
                        min_ask = price_raw;
                    }
                    if &owner_address == orders_key {
                        open_asks.push(order_id);
                    }
                }
                None => {
                    break;
                }
            }
        }
    }

    println!("open bids: {:?}", &open_bids);
    println!("open asks: {:?}", &open_asks);

    Ok(OpenOrderState{
        min_ask,
        max_bid,
        open_asks,
        open_bids,
        bids_address,
        asks_address,
    })
}
