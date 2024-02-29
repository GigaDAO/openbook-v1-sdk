use std::str::FromStr;
use openbook_dex::state::MarketState;
use solana_program::pubkey::Pubkey;
use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::signature::Signer;
use crate::{ObClient, OPENBOOK_V1_PROGRAM_ID, SOL_USDC_MARKET_ID};
use crate::load_oo_state::load_oo_state;
use crate::utils::{create_account_info_from_account, read_keypair};

pub fn load_ob_client() -> anyhow::Result<ObClient>{

    // load env
    dotenv::dotenv().ok();
    let key_path = std::env::var("KEY_PATH").expect("KEY_PATH is not set in .env file");
    let keypair = read_keypair(&key_path);
    let OOS_KEY_STR = std::env::var("OOS_KEY").expect("OOS_KEY is not set in .env file");
    let orders_key = Pubkey::from_str(OOS_KEY_STR.as_str())?;
    let usdc_ata_str = std::env::var("USDC_ATA").expect("USDC_ATA is not set in .env file");
    let usdc_ata = Pubkey::from_str(usdc_ata_str.as_str()).unwrap();
    let wsol_ata_str = std::env::var("WSOL_ATA").expect("WSOL_ATA is not set in .env file");
    let wsol_ata = Pubkey::from_str(wsol_ata_str.as_str()).unwrap();
    let rpc_url = std::env::var("RPC_URL").expect("RPC_URL is not set in .env file");

    // load state
    let mut rpc_client = RpcClient::new(rpc_url);
    let mut account = rpc_client.get_account(&SOL_USDC_MARKET_ID.parse().unwrap())?;
    let program_id_binding = OPENBOOK_V1_PROGRAM_ID.parse().unwrap();
    let market_account_binding = SOL_USDC_MARKET_ID.parse().unwrap();
    let account_info = create_account_info_from_account(&mut account, &market_account_binding, &program_id_binding, false, false);
    let oo_state;
    {
        let market_state = MarketState::load(
            &account_info,
            &OPENBOOK_V1_PROGRAM_ID.parse().unwrap(),
            false,
        )?;
        oo_state = load_oo_state(&mut rpc_client, market_state, &orders_key)?;
    }

    let ob_client;
    {
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

        // let order_id_0 = oos.orders[0];
        // println!("order id 0: {order_id_0}");
        // let base_total = oos.native_coin_total;
        // let quote_total = oos.native_pc_total;
        // println!("base total: {base_total}, quote total: {quote_total}");
        // let base_free = oos.native_coin_free;
        // let quote_free = oos.native_pc_free;
        // println!("base free: {base_free}, quote free: {quote_free}");
        //
        // let wsol_total = base_total as f64 / 1e9;
        // let usdc_total = quote_total as f64 / 1e6;
        // println!("WSOL: {:6.4}", wsol_total);
        // println!("USDC: {:6.4}", usdc_total);

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

        ob_client = ObClient {
            rpc_client: debug_ignore::DebugIgnore(rpc_client),
            market_account: market_account_binding,
            open_orders_account: orders_key,
            request_queue,
            event_queue,
            base_ata: wsol_ata,
            quote_ata: usdc_ata,
            keypair,
            coin_vault,
            pc_vault,
            program_id: program_id_binding,
            vault_signer_key,
            oo_state,
        }

    }

    Ok(ob_client)

}