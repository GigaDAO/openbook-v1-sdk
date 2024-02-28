use std::cell::RefMut;
use std::convert::identity;
use std::str::FromStr;
use openbook_dex::critbit::{SlabView};
use openbook_dex::state::{MarketState, ToAlignedBytes};
use solana_program::{
    account_info::AccountInfo,
    pubkey::Pubkey,
};
use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::account::Account;

const SOL_USDC_MARKET_ID: &str = "8BnEgHoWFysVcuFFX7QztDmzuH8r5ZFvyP3sYwn1XTh6";
const OPENBOOK_V1_PROGRAM_ID: &str = "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX";


pub fn load_market() -> anyhow::Result<()>{
    dotenv::dotenv().ok();
    let rpc_url = std::env::var("RPC_URL").expect("RPC_URL is not set in .env file");
    let rpc_client = RpcClient::new(rpc_url);
    let mut account = rpc_client.get_account(&SOL_USDC_MARKET_ID.parse().unwrap())?;
    let program_id_binding = OPENBOOK_V1_PROGRAM_ID.parse().unwrap();
    let market_account_binding = SOL_USDC_MARKET_ID.parse().unwrap();
    // let binding = OPENBOOK_V1_PROGRAM_ID.parse().unwrap();
    let account_info = create_account_info_from_account(&mut account, &market_account_binding, &program_id_binding, false, false);
    let market_state = openbook_dex::state::MarketState::load(
        &account_info,
        &OPENBOOK_V1_PROGRAM_ID.parse().unwrap(),
        false,
    )?;
    println!("got market state: {:?}", &market_state);

    let (bids_address, asks_address) = get_bids_asks_addresses(&market_state);

    let mut bids_account = rpc_client.get_account(&bids_address)?;
    let mut bids_info = create_account_info_from_account(&mut bids_account, &bids_address, &program_id_binding, false, false);
    let mut bids = market_state.load_bids_mut(&mut bids_info)?;

    loop {
        let node = bids.remove_max();
        match node {
            Some(node) => {
                println!("got node: {:?}", node);
            }
            None => {
                break;
            }
        }
    }

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        load_market().unwrap();
    }
}
