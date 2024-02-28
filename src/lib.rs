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
    let binding = OPENBOOK_V1_PROGRAM_ID.parse().unwrap();
    let account_info = create_account_info_from_account(&mut account, &binding, false, false);
    let market_state = openbook_dex::state::MarketState::load(
        &account_info,
        &OPENBOOK_V1_PROGRAM_ID.parse().unwrap(),
        false,
    );

    println!("got market state: {:?}", market_state);


    Ok(())
}

pub fn create_account_info_from_account<'a>(
    account: &'a mut Account,
    my_program_id: &'a Pubkey,
    is_signer: bool,
    is_writable: bool
) -> AccountInfo<'a> {
    AccountInfo::new(
        my_program_id,
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
