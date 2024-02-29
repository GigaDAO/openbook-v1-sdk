/*
COIN == BASE
PC == QUOTE
 */
use std::cell::RefMut;
use std::fs;
use std::num::NonZeroU64;
use std::ops::DerefMut;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use openbook_dex::critbit::{SlabView};
use openbook_dex::instruction::SelfTradeBehavior;
use openbook_dex::matching::{OrderType, Side};
use openbook_dex::state::{MarketState, ToAlignedBytes};
use solana_program::{
    account_info::AccountInfo,
    pubkey::Pubkey,
};
use solana_program::instruction::Instruction;
use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::bs58;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use rand::random;
use solana_rpc_client_api::config::RpcSendTransactionConfig;

const SOL_USDC_MARKET_ID: &str = "8BnEgHoWFysVcuFFX7QztDmzuH8r5ZFvyP3sYwn1XTh6";
const OPENBOOK_V1_PROGRAM_ID: &str = "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX";


pub fn load_market() -> anyhow::Result<()>{

    // config
    let place_limit_bid = false;
    let cancel_order = false;
    let settle_balance = false;


    dotenv::dotenv().ok();

    let key_path = std::env::var("KEY_PATH").expect("KEY_PATH is not set in .env file");
    let keypair = read_keypair(&key_path);

    let OOS_KEY_STR = std::env::var("OOS_KEY").expect("OOS_KEY is not set in .env file");
    let orders_key = Pubkey::from_str(OOS_KEY_STR.as_str())?;

    let rpc_url = std::env::var("RPC_URL").expect("RPC_URL is not set in .env file");
    let rpc_client = RpcClient::new(rpc_url);
    let mut account = rpc_client.get_account(&SOL_USDC_MARKET_ID.parse().unwrap())?;
    let program_id_binding = OPENBOOK_V1_PROGRAM_ID.parse().unwrap();
    let market_account_binding = SOL_USDC_MARKET_ID.parse().unwrap();
    let account_info = create_account_info_from_account(&mut account, &market_account_binding, &program_id_binding, false, false);

    let coin_decimals = 9; // base (WSOL)
    let pc_decimals = 6; // quote (USDC)
    let coin_lot_size = 1_000_000;
    let pc_lot_size = 1;

    let bids_address;
    let asks_address;

    let mut max_bid = 0;
    {
        let market_state = MarketState::load(
            &account_info,
            &OPENBOOK_V1_PROGRAM_ID.parse().unwrap(),
            false,
        )?;
        (bids_address, asks_address) = get_bids_asks_addresses(&market_state);
        let mut bids_account = rpc_client.get_account(&bids_address)?;
        let mut bids_info = create_account_info_from_account(&mut bids_account, &bids_address, &program_id_binding, false, false);
        let mut bids = market_state.load_bids_mut(&mut bids_info)?;
        loop {
            let node = bids.remove_max();
            match node {
                Some(node) => {

                    let owner_byes = node.owner();
                    //
                    let mut bytes: [u8; 32] = [0; 32];
                    for i in 0..4 {
                        bytes[i*8..i*8+8].copy_from_slice(&owner_byes[i].to_le_bytes());
                    }
                    let owner_address = Pubkey::from(bytes);
                    let order_id = node.order_id();
                    let price_raw = node.price().get();
                    let price = price_raw as f64 / 1e3;

                    if max_bid == 0 {
                        max_bid = price_raw;
                    }

                    // println!("{order_id}, {}", owner_address.to_string());

                    if &owner_address == &orders_key {
                        println!("FOUND oid: {order_id}, price: {price}");
                    }

                }
                None => {
                    break;
                }
            }
        }
    }


    // NOTE get open orders - must cross reference with loaded bids/asks
    let mut orders_account = rpc_client.get_account(&orders_key)?;
    let orders_account_info = create_account_info_from_account(&mut orders_account, &orders_key, &program_id_binding, false, false);

    let mut owners_account = rpc_client.get_account(&keypair.pubkey())?;
    let binding = keypair.pubkey();
    let owners_account_info = create_account_info_from_account(&mut owners_account, &binding, &program_id_binding, false, false);

    let market = openbook_dex::state::Market::load(&account_info, &program_id_binding, false)?;

    let vault_signer_nonce = market.vault_signer_nonce;
    let vault_signer_key = openbook_dex::state::gen_vault_signer_key(vault_signer_nonce, &market_account_binding, &program_id_binding)?;

    let oos = market.load_orders_mut(
        &orders_account_info,
        Some(&owners_account_info),
        &program_id_binding,
        None,
        None,
    )?;

    let order_id_0 = oos.orders[0];
    println!("order id 0: {order_id_0}");

    let free_slots = oos.free_slot_bits;
    println!("free slots: {free_slots}");

    let base_total = oos.native_coin_total;
    let quote_total = oos.native_pc_total;
    println!("base total: {base_total}, quote total: {quote_total}");
    let base_free = oos.native_coin_free;
    let quote_free = oos.native_pc_free;
    println!("base free: {base_free}, quote free: {quote_free}");

    let wsol_total = base_total as f64 / 1e9;
    let usdc_total = quote_total as f64 / 1e6;
    println!("WSOL: {:6.4}", wsol_total);
    println!("USDC: {:6.4}", usdc_total);

    // load keys (from u64 arr)
    let request_queue;
    let event_queue;
    let coin_vault;
    let pc_vault;
    //
    let mut bytes: [u8; 32] = [0; 32];
    for i in 0..4 {
        bytes[i*8..i*8+8].copy_from_slice(&market.req_q[i].to_le_bytes());
    }
    request_queue = Pubkey::from(bytes);
    //
    let mut bytes: [u8; 32] = [0; 32];
    for i in 0..4 {
        bytes[i*8..i*8+8].copy_from_slice(&market.event_q[i].to_le_bytes());
    }
    event_queue = Pubkey::from(bytes);
    //
    let mut bytes: [u8; 32] = [0; 32];
    for i in 0..4 {
        bytes[i*8..i*8+8].copy_from_slice(&market.coin_vault[i].to_le_bytes());
    }
    coin_vault = Pubkey::from(bytes);
    //
    let mut bytes: [u8; 32] = [0; 32];
    for i in 0..4 {
        bytes[i*8..i*8+8].copy_from_slice(&market.pc_vault[i].to_le_bytes());
    }
    pc_vault = Pubkey::from(bytes);

    // NOTE: all math below is hard coded, should be dynamic using decimals and lot sizes
    let coin_lot_size = market.coin_lot_size;
    let pc_lot_size = market.pc_lot_size;
    // println!("coin lot size: {coin_lot_size}");
    // println!("pc lot size: {pc_lot_size}");

    let target_size_usdc = 1.0;
    let target_usdc_lots_w_fee = (target_size_usdc * 1e6 * 1.1) as u64;
    // let price = max_bid as f64 / 1e3;
    let price = max_bid as f64 / 1e3 - 30.;
    let new_bid = (price * 1e3) as u64;
    // println!("target pride: {price}");
    let target_amount_wsol = target_size_usdc / price;
    let target_wsol_lots = (target_amount_wsol * 1e3) as u64;
    // println!("using target wsol lots: {target_wsol_lots}");

    let limit_price = NonZeroU64::new(new_bid).unwrap();
    let max_coin_qty = NonZeroU64::new(target_wsol_lots).unwrap(); // max wsol lots
    let max_native_pc_qty_including_fees = NonZeroU64::new(target_usdc_lots_w_fee).unwrap(); // max usdc lots + fees

    //
    let limit = u16::MAX;
    let client_order_id = random::<u64>();
    // println!("client order id: {client_order_id}");

    let usdc_ata_str = std::env::var("USDC_ATA").expect("USDC_ATA is not set in .env file");
    let usdc_ata = Pubkey::from_str(usdc_ata_str.as_str()).unwrap();

    let wsol_ata_str = std::env::var("WSOL_ATA").expect("WSOL_ATA is not set in .env file");
    let wsol_ata = Pubkey::from_str(wsol_ata_str.as_str()).unwrap();

    if place_limit_bid {
        let place_order_ix = openbook_dex::instruction::new_order(
            &market_account_binding,
            &orders_key,
            &request_queue,
            &event_queue,
            &bids_address,
            &asks_address,
            &usdc_ata,
            &keypair.pubkey(),
            &coin_vault,
            &pc_vault,
            &anchor_spl::token::ID,
            &solana_program::sysvar::rent::ID,
            None,
            &program_id_binding,
            Side::Bid,
            limit_price,
            max_coin_qty,
            OrderType::PostOnly,
            client_order_id,
            SelfTradeBehavior::AbortTransaction,
            limit,
            max_native_pc_qty_including_fees,
            (get_unix_secs() + 30) as i64,
        )?;

        // place order
        let mut instructions = Vec::new();
        instructions.push(place_order_ix);

        let recent_hash = rpc_client.get_latest_blockhash()?;
        let txn = Transaction::new_signed_with_payer(
            &instructions,
            Some(&keypair.pubkey()),
            &[&keypair],
            recent_hash,
        );

        let mut config = RpcSendTransactionConfig::default();
        config.skip_preflight = true;
        let r = rpc_client.send_transaction_with_config(&txn, config);
        println!("got results: {:?}", r);
    }

    if cancel_order {
        let ix = openbook_dex::instruction::cancel_order(
            &program_id_binding,
            &market_account_binding,
            &bids_address,
            &asks_address,
            &orders_key,
            &keypair.pubkey(),
            &event_queue,
            Side::Bid,
            order_id_0,
        )?;
        // place order
        let mut instructions = Vec::new();
        instructions.push(ix);

        let recent_hash = rpc_client.get_latest_blockhash()?;
        let txn = Transaction::new_signed_with_payer(
            &instructions,
            Some(&keypair.pubkey()),
            &[&keypair],
            recent_hash,
        );

        let mut config = RpcSendTransactionConfig::default();
        config.skip_preflight = true;
        let r = rpc_client.send_transaction_with_config(&txn, config);
        println!("got results: {:?}", r);
    }

    if settle_balance {

        let ix = openbook_dex::instruction::settle_funds(
            &program_id_binding,
            &market_account_binding,
            &anchor_spl::token::ID,
            &orders_key,
            &keypair.pubkey(),
            &coin_vault,
            &wsol_ata,
            &pc_vault,
            &usdc_ata,
            None,
            &vault_signer_key,
        )?;

        // place order
        let mut instructions = Vec::new();
        instructions.push(ix);

        let recent_hash = rpc_client.get_latest_blockhash()?;
        let txn = Transaction::new_signed_with_payer(
            &instructions,
            Some(&keypair.pubkey()),
            &[&keypair],
            recent_hash,
        );

        let mut config = RpcSendTransactionConfig::default();
        config.skip_preflight = true;
        let r = rpc_client.send_transaction_with_config(&txn, config);
        println!("got results: {:?}", r);

    }


    Ok(())

}

fn _initialize_new_oos_account() {
    // let (orders_key, create_oos_account_ix) = create_dex_account(&rpc_client, &program_id_binding, &keypair.pubkey(), size_of::<openbook_dex::state::OpenOrders>())?;
    // let init_ix = openbook_dex::instruction::init_open_orders(
    //     &program_id_binding,
    //     &orders_key.pubkey(),
    //     &keypair.pubkey(),
    //     &market_account_binding,
    //     None,
    // )?;
    // println!("got oos account: {:?}", orders_key.pubkey());
    //
    // let mut instructions = Vec::new();
    // instructions.push(create_oos_account_ix);
    // instructions.push(init_ix);
    //
    // let recent_hash = rpc_client.get_latest_blockhash()?;
    // let txn = Transaction::new_signed_with_payer(
    //     &instructions,
    //     Some(&keypair.pubkey()),
    //     &[&orders_key, &keypair],
    //     recent_hash,
    // );
    //
    // let r = rpc_client.send_transaction(&txn);
    // println!("got results: {:?}", r);
}

fn create_dex_account(
    client: &RpcClient,
    program_id: &Pubkey,
    payer: &Pubkey,
    unpadded_len: usize,
) -> anyhow::Result<(Keypair, Instruction)> {
    let len = unpadded_len + 12;
    let key = Keypair::new();
    let create_account_instr = solana_sdk::system_instruction::create_account(
        payer,
        &key.pubkey(),
        client.get_minimum_balance_for_rent_exemption(len)?,
        len as u64,
        program_id,
    );
    Ok((key, create_account_instr))
}

fn get_bids_asks_addresses(market_state: &RefMut<MarketState>) -> (Pubkey, Pubkey) {

    let mut bytes: [u8; 32] = [0; 32];
    for i in 0..4 {
        bytes[i*8..i*8+8].copy_from_slice(&market_state.bids[i].to_le_bytes());
    }
    let bids_address = Pubkey::from(bytes);

    let mut bytes: [u8; 32] = [0; 32];
    for i in 0..4 {
        bytes[i*8..i*8+8].copy_from_slice(&market_state.asks[i].to_le_bytes());
    }
    let asks_address = Pubkey::from(bytes);

    (bids_address, asks_address)
}


fn create_account_info_from_account<'a>(
    account: &'a mut Account,
    key: &'a Pubkey,
    my_program_id: &'a Pubkey,
    is_signer: bool,
    is_writable: bool
) -> AccountInfo<'a> {
    AccountInfo::new(
        key,
        is_signer,
        is_writable,
        &mut account.lamports,
        &mut account.data,
        &my_program_id,
        account.executable,
        account.rent_epoch,
    )
}

fn read_keypair(path: &String) -> Keypair {
    let secret_string: String = fs::read_to_string(path).expect("Can't find key file");
    let secret_bytes: Vec<u8> = match serde_json::from_str(&secret_string) {
        Ok(bytes) => bytes,
        Err(_) => match bs58::decode(&secret_string.trim()).into_vec() {
            Ok(bytes) => bytes,
            Err(_) => panic!("failed to load secret key from file"),
        },
    };
    let keypair = Keypair::from_bytes(&secret_bytes).expect("failed to generate keypair from secret bytes");
    keypair
}

pub fn get_unix_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        load_market().unwrap();
    }
}
