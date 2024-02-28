use std::cell::RefMut;
use std::fs;
use std::mem::size_of;
use std::str::FromStr;
use openbook_dex::critbit::{SlabView};
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

const SOL_USDC_MARKET_ID: &str = "8BnEgHoWFysVcuFFX7QztDmzuH8r5ZFvyP3sYwn1XTh6";
const OPENBOOK_V1_PROGRAM_ID: &str = "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX";


pub fn load_market() -> anyhow::Result<()>{
    dotenv::dotenv().ok();
    let rpc_url = std::env::var("RPC_URL").expect("RPC_URL is not set in .env file");
    let rpc_client = RpcClient::new(rpc_url);
    let mut account = rpc_client.get_account(&SOL_USDC_MARKET_ID.parse().unwrap())?;
    let program_id_binding = OPENBOOK_V1_PROGRAM_ID.parse().unwrap();
    let market_account_binding = SOL_USDC_MARKET_ID.parse().unwrap();
    let account_info = create_account_info_from_account(&mut account, &market_account_binding, &program_id_binding, false, false);
    let market_state = openbook_dex::state::MarketState::load(
        &account_info,
        &OPENBOOK_V1_PROGRAM_ID.parse().unwrap(),
        false,
    )?;
    let (bids_address, asks_address) = get_bids_asks_addresses(&market_state);
    let mut bids_account = rpc_client.get_account(&bids_address)?;
    let mut bids_info = create_account_info_from_account(&mut bids_account, &bids_address, &program_id_binding, false, false);
    let mut bids = market_state.load_bids_mut(&mut bids_info)?;
    loop {
        let node = bids.remove_max();
        match node {
            Some(_node) => {
                // println!("got node: {:?}", node);
            }
            None => {
                break;
            }
        }
    }
    let key_path = std::env::var("KEY_PATH").expect("RPC_URL is not set in .env file");
    let keypair = read_keypair(&key_path);

    let (orders_key, create_oos_account_ix) = create_dex_account(&rpc_client, &program_id_binding, &keypair.pubkey(), size_of::<openbook_dex::state::OpenOrders>())?;
    let init_ix = openbook_dex::instruction::init_open_orders(
        &program_id_binding,
        &orders_key.pubkey(),
        &keypair.pubkey(),
        &market_account_binding,
        None,
    )?;
    println!("got oos account: {:?}", orders_key.pubkey());

    let mut instructions = Vec::new();
    instructions.push(create_oos_account_ix);
    instructions.push(init_ix);

    let recent_hash = rpc_client.get_latest_blockhash()?;
    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&keypair.pubkey()),
        &[&orders_key, &keypair],
        recent_hash,
    );

    let r = rpc_client.send_transaction(&txn);
    println!("got results: {:?}", r);

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        load_market().unwrap();
    }
}
