use std::ops::DerefMut;
use std::str::FromStr;
use openbook_dex::state::MarketState;
use solana_program::pubkey::Pubkey;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::Signer;
use crate::{ObClient, OPENBOOK_V1_PROGRAM_ID};
use crate::initialize_oo_account::initialize_new_oos_account;
use crate::load_oo_state::load_oo_state;
use crate::utils::{create_account_info_from_account, read_keypair};

pub struct EnvConfig {
    pub key_path: String,
    pub oos_key: String,
    pub usdc_ata: String,
    pub wsol_ata: String,
}

fn load_test_env() -> EnvConfig {
    // load env
    dotenv::dotenv().ok();
    let key_path = std::env::var("KEY_PATH").expect("KEY_PATH is not set in .env file");
    let OOS_KEY_STR = std::env::var("OOS_KEY").expect("OOS_KEY is not set in .env file");
    let usdc_ata_str = std::env::var("USDC_ATA").expect("USDC_ATA is not set in .env file");
    let wsol_ata_str = std::env::var("WSOL_ATA").expect("WSOL_ATA is not set in .env file");
    EnvConfig {
        key_path,
        oos_key: OOS_KEY_STR,
        usdc_ata: usdc_ata_str,
        wsol_ata: wsol_ata_str,
    }
}

pub enum LoadResult {
    Client(ObClient),
    OpenOrdersAddress(Pubkey),
}

pub async fn load_ob_client(conf: Option<EnvConfig>) -> anyhow::Result<LoadResult>{

    let mut config;
    if let Some(conf) = conf {
        config = conf;
    }
    else {
        config = load_test_env();
    }

    /*
    Solutions:
    1) keep in one thread
    2) use Arc<Mutex
     */

    // load env
    dotenv::dotenv().ok();
    let keypair = read_keypair(&config.key_path);
    let rpc_url = std::env::var("RPC_URL").expect("RPC_URL is not set in .env file");
    let mut rpc_client = RpcClient::new(rpc_url);

    let market_id = std::env::var("MARKET_ID").expect("MARKET_ID is not set in .env file");
    let market_account_binding = market_id.as_str().parse().unwrap();
    let mut account = rpc_client.get_account(&market_account_binding).await?;
    let mut account_2 = account.clone();
    let program_id_binding = OPENBOOK_V1_PROGRAM_ID.parse().unwrap();

    let orders_key = Pubkey::from_str(config.oos_key.as_str());

    let mut actual_oos;
    if orders_key.is_err() {
        println!("initializing new...");
        let pk = initialize_new_oos_account(&mut rpc_client, program_id_binding, &keypair, market_account_binding).await?;
        println!("initialized...");
        actual_oos = pk;
        return Ok(LoadResult::OpenOrdersAddress(actual_oos));
    }
    else {
        actual_oos = orders_key.unwrap();
    }

    let orders_key = actual_oos;

    let usdc_ata = Pubkey::from_str(config.usdc_ata.as_str()).unwrap();
    let wsol_ata = Pubkey::from_str(config.wsol_ata.as_str()).unwrap();


    let ob_client;
    {
        let mut orders_account = rpc_client.get_account(&orders_key).await?;
        let mut owners_account = rpc_client.get_account(&keypair.pubkey()).await?;


        let binding = keypair.pubkey();
        let base_total;
        let quote_total;
        let base_free;
        let quote_free;
        let wsol_total;
        let usdc_total;

        let request_queue;
        let event_queue;
        let coin_vault;
        let pc_vault;
        let claimable;
        let vault_signer_key;

        {
            let account_info_2 = create_account_info_from_account(&mut account_2, &market_account_binding, &program_id_binding, false, false);
            let market = openbook_dex::state::Market::load(&account_info_2, &program_id_binding, false)?;

            let vault_signer_nonce = market.vault_signer_nonce;
            vault_signer_key = openbook_dex::state::gen_vault_signer_key(vault_signer_nonce, &market_account_binding, &program_id_binding)?;


            {
                let orders_account_info = create_account_info_from_account(&mut orders_account, &orders_key, &program_id_binding, false, false);
                let owners_account_info = create_account_info_from_account(&mut owners_account, &binding, &program_id_binding, false, false);
                let mut oos = market.load_orders_mut(
                    &orders_account_info,
                    Some(&owners_account_info),
                    &program_id_binding,
                    None,
                    None,
                )?.deref_mut().clone();

                base_total = oos.native_coin_total;
                quote_total = oos.native_pc_total;
                base_free = oos.native_coin_free;
                quote_free = oos.native_pc_free;
                wsol_total = base_total as f64 / 1e6;
                usdc_total = quote_total as f64 / 1e6;
            }


            claimable = base_free > 0 || quote_free > 0;

            tracing::debug!("base total: {base_total}, quote total: {quote_total}");
            tracing::debug!("base free: {base_free}, quote free: {quote_free}");
            tracing::debug!("BASE (JLP): {:6.4}", wsol_total);
            // TODO off by 10 (is 10 instead of 100)
            tracing::debug!("QUOTE(USDC): {:6.4}", usdc_total);

            // load keys (from u64 arr)
           //
            let mut bytes: [u8; 32] = [0; 32];
            for i in 0..4 {
                bytes[i * 8..i * 8 + 8].copy_from_slice(&market.req_q[i].to_le_bytes());
            }
            request_queue = Pubkey::from(bytes);
            //
            let mut bytes: [u8; 32] = [0; 32];
            for i in 0..4 {
                bytes[i * 8..i * 8 + 8].copy_from_slice(&market.event_q[i].to_le_bytes());
            }
            event_queue = Pubkey::from(bytes);
            //
            let mut bytes: [u8; 32] = [0; 32];
            for i in 0..4 {
                bytes[i * 8..i * 8 + 8].copy_from_slice(&market.coin_vault[i].to_le_bytes());
            }
            coin_vault = Pubkey::from(bytes);
            //
            let mut bytes: [u8; 32] = [0; 32];
            for i in 0..4 {
                bytes[i * 8..i * 8 + 8].copy_from_slice(&market.pc_vault[i].to_le_bytes());
            }
            pc_vault = Pubkey::from(bytes);

        }

        let mut oo_state;
        {
            let mut market_state;
            {
                let account_info = create_account_info_from_account(&mut account, &market_account_binding, &program_id_binding, false, false);
                market_state = MarketState::load(
                    &account_info,
                    &OPENBOOK_V1_PROGRAM_ID.parse().unwrap(),
                    false,
                )?.deref_mut().clone();
            }
            oo_state = load_oo_state(&mut rpc_client, market_state, &orders_key).await?;
        }
        oo_state.base_total = wsol_total;
        oo_state.quote_total = usdc_total;



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
            claimable,
        }

    }

    Ok(LoadResult::Client(ob_client))

}