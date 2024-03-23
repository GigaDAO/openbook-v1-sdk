use std::num::NonZeroU64;
use openbook_dex::instruction::SelfTradeBehavior;
use openbook_dex::matching::{OrderType, Side};
use rand::random;
use solana_program::instruction::Instruction;
use solana_rpc_client_api::config::RpcSendTransactionConfig;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::Transaction;
use crate::ObClient;
use crate::utils::get_unix_secs;

pub async fn place_limit_order(
    ob_client: &mut ObClient,
    target_amount_quote: f64,
    side: Side,
    target_price: f64,
) -> anyhow::Result<Instruction> {

    // j/usdc
    let base_d_factor = 1e6;
    let quote_d_factor = 1e6;
    let base_lot_factor = 1e5;
    let quote_lot_factor = 1e1;

    let price_factor = quote_d_factor * base_lot_factor / base_d_factor / quote_lot_factor;

    let (input_ata, price) = match side {
        Side::Bid => {
            let price = target_price;
            (&ob_client.quote_ata, price)
        }
        Side::Ask => {
            let price = target_price;
            (&ob_client.base_ata, price)
        }
    };

    let limit_price_lots = (price * price_factor) as u64;
    let target_amount_base = target_amount_quote / price;

    let target_base_lots = (target_amount_base * base_d_factor / base_lot_factor) as u64;
    let target_quote_lots_w_fee = (target_base_lots as f64 * quote_lot_factor * limit_price_lots as f64)  as u64;


    let limit_price = NonZeroU64::new(limit_price_lots).unwrap();
    let max_coin_qty = NonZeroU64::new(target_base_lots).unwrap(); // max wsol lots
    let max_native_pc_qty_including_fees = NonZeroU64::new(target_quote_lots_w_fee).unwrap(); // max usdc lots + fees

    let place_order_ix = openbook_dex::instruction::new_order(
        &ob_client.market_account,
        &ob_client.open_orders_account,
        &ob_client.request_queue,
        &ob_client.event_queue,
        &ob_client.bids_address,
        &ob_client.asks_address,
        input_ata,
        &ob_client.keypair.pubkey(),
        &ob_client.coin_vault,
        &ob_client.pc_vault,
        &spl_token::ID,
        &solana_program::sysvar::rent::ID,
        None,
        &ob_client.program_id,
        side,
        limit_price, // price number to lots (this works but gotta be dynamic)
        max_coin_qty, // base quantity in lots
        OrderType::PostOnly,
        random::<u64>(),
        SelfTradeBehavior::AbortTransaction,
        u16::MAX,
        max_native_pc_qty_including_fees, // base_lots * price_lots * quote lots
        (get_unix_secs() + 30) as i64,
    )?;

    Ok(place_order_ix)

}
