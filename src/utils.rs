use std::cell::RefMut;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use openbook_dex::state::MarketState;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;
use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::bs58;
use solana_sdk::signature::{Keypair, Signer};
use crate::ObClient;

pub fn create_dex_account(
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

pub fn get_bids_asks_addresses(market_state: &RefMut<MarketState>) -> (Pubkey, Pubkey) {

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


pub fn create_account_info_from_account<'a>(
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

pub fn read_keypair(path: &String) -> Keypair {
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

pub fn get_equity(ob_client: &mut ObClient) -> anyhow::Result<(f64, f64)>{

    let ba = &ob_client.base_ata;
    let qa = &ob_client.quote_ata;

    let bb = ob_client.rpc_client.get_token_account_balance(ba)?.ui_amount.unwrap();
    let qb = ob_client.rpc_client.get_token_account_balance(qa)?.ui_amount.unwrap();

    Ok((bb, qb))

}



