/*
COIN == BASE
PC == QUOTE
 */
pub mod utils;
pub mod ixs;

pub use openbook_dex::matching::Side;
use std::ops::DerefMut;
use std::str::FromStr;
use openbook_dex::critbit::{SlabView};
use solana_program::{
    pubkey::Pubkey,
};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::Keypair;
use crate::utils::read_keypair;

const OPENBOOK_V1_PROGRAM_ID: &str = "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX";

pub struct ObClient {
    pub market_account: Pubkey,
    pub open_orders_account: Pubkey,
    pub request_queue: Pubkey,
    pub event_queue: Pubkey,
    pub base_ata: Pubkey,
    pub quote_ata: Pubkey,
    pub coin_vault: Pubkey, // base
    pub pc_vault: Pubkey, //quote
    pub program_id: Pubkey,
    pub vault_signer_key: Pubkey,
    pub bids_address: Pubkey,
    pub asks_address: Pubkey,
    pub rpc_client: RpcClient,
    pub keypair: Keypair,
}

impl ObClient {
    pub fn load_hard_coded() -> Self {
        dotenv::dotenv().ok();
        let triton_url = std::env::var("RPC_URL").unwrap();
        let rpc_client = RpcClient::new_with_commitment(triton_url, CommitmentConfig::confirmed());
        let keypair = read_keypair(&std::env::var("KEY_PATH").unwrap());
        Self {
            rpc_client,
            keypair,
            bids_address: Pubkey::from_str("E9jHtpUqgTF2Ln8UhmyRXRNJsGKuNMVaSVaGowk9Vvr6").unwrap(),
            asks_address: Pubkey::from_str("6Kus1PbGpDRZ8R57PG2UM5b5vmyMp9wAHsXzsFQfPzsZ").unwrap(),
            market_account: Pubkey::from_str("ASUyMMNBpFzpW3zDSPYdDVggKajq1DMKFFPK1JS9hoSR").unwrap(),
            open_orders_account: Pubkey::from_str("Cg9qSVSoqCgmzk7sf76fqV5naxb2kHcBncSiGuYZCiyg").unwrap(),
            request_queue: Pubkey::from_str("7oGLtLJbcaTWQprDoYyCBTUW5n598qYRQP6KKw5DML4L").unwrap(),
            event_queue:  Pubkey::from_str("FM1a4He7jBDBQXfbUK35xpwf6tx2DfRYAzX48AkVcNqP").unwrap(),
            base_ata:  Pubkey::from_str("9vTKnac15XTDYDpxcXEncBRwJ6Ctn5crD11deHKDTqn").unwrap(),
            quote_ata: Pubkey::from_str("2AuYk75jwcGBdqtdJiZN61oSqLEPbYwsh4XkgEjFMNiE").unwrap(),
            coin_vault: Pubkey::from_str("BNSehB5QgfqfUGa8N2GW7qwjGQD8sUjNUzTSEcmmupZL").unwrap(),
            pc_vault:  Pubkey::from_str("HstJUT5jehxW29UMUjockEWTVhwUhwxk1WQhHzPcZJXt").unwrap(),
            program_id: Pubkey::from_str("srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX").unwrap(),
            vault_signer_key:  Pubkey::from_str("G26Hizvx9zttK3Nu3n9oQouEoK89aeSqUQw6AKx4oWic").unwrap(),
        }
    }
}


// pub async fn test_place_and_cancel() -> anyhow::Result<()>{
//
//     let mut ob_client = load_ob_client(None).await?;
//
//     match ob_client {
//         LoadResult::Client(client) => {
//             println!("got client: {:?}", client);
//         }
//         LoadResult::OpenOrdersAddress(_) => {}
//     }
//
//     Ok(())
// }
//
//
//
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[tokio::test]
//     async fn it_works() {
//         test_place_and_cancel().await.unwrap();
//
//     }
// }
