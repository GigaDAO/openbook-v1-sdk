use std::num::NonZeroU64;
use openbook_dex::instruction::SelfTradeBehavior;
use openbook_dex::matching::{OrderType, Side};
use rand::random;
use solana_rpc_client_api::config::RpcSendTransactionConfig;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::Transaction;
use crate::ObClient;
use crate::utils::get_unix_secs;

pub fn place_limit_order(
    ob_client: &mut ObClient,
    target_size_usdc: f64,
    side: Side,
    best_offset_usdc: f64,
) -> anyhow::Result<()> {


    // TODO dynamic
    // wsol/usdc
    // let base_d_factor = 1e9;
    // let quote_d_factor = 1e6;
    // let base_lot_factor = 1e6;
    // let quote_lot_factor = 1e0;

    // j/usdc
    let base_d_factor = 1e6;
    let quote_d_factor = 1e6;
    let base_lot_factor = 1e5;
    let quote_lot_factor = 1e1;

    let price_factor = quote_d_factor * base_lot_factor / base_d_factor / quote_lot_factor;
    let target_usdc_lots_w_fee = (target_size_usdc * quote_d_factor * 1.1) as u64; // NOTE should be negative fees...

    let (input_ata, price) = match side {
        Side::Bid => {

            let price = ob_client.oo_state.max_bid as f64 / price_factor - best_offset_usdc;

            (&ob_client.quote_ata, price)
        }
        Side::Ask => {

            let price = ob_client.oo_state.min_ask as f64 / price_factor + best_offset_usdc;

            (&ob_client.base_ata, price)
        }
    };

    let new_bid = (price * price_factor) as u64;
    let target_amount_wsol = target_size_usdc / price;
    let target_wsol_lots = (target_amount_wsol * base_d_factor / base_lot_factor) as u64;
    let limit_price = NonZeroU64::new(new_bid).unwrap();
    let max_coin_qty = NonZeroU64::new(target_wsol_lots).unwrap(); // max wsol lots
    let max_native_pc_qty_including_fees = NonZeroU64::new(target_usdc_lots_w_fee).unwrap(); // max usdc lots + fees

    let place_order_ix = openbook_dex::instruction::new_order(
        &ob_client.market_account,
        &ob_client.open_orders_account,
        &ob_client.request_queue,
        &ob_client.event_queue,
        &ob_client.oo_state.bids_address,
        &ob_client.oo_state.asks_address,
        input_ata,
        &ob_client.keypair.pubkey(),
        &ob_client.coin_vault,
        &ob_client.pc_vault,
        &anchor_spl::token::ID,
        &solana_program::sysvar::rent::ID,
        None,
        &ob_client.program_id,
        side,
        limit_price,
        max_coin_qty,
        OrderType::PostOnly,
        random::<u64>(),
        SelfTradeBehavior::AbortTransaction,
        u16::MAX,
        max_native_pc_qty_including_fees,
        (get_unix_secs() + 30) as i64,
    )?;

    // place order
    let mut instructions = Vec::new();
    instructions.push(place_order_ix);

    let recent_hash = ob_client.rpc_client.get_latest_blockhash()?;
    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&ob_client.keypair.pubkey()),
        &[&ob_client.keypair],
        recent_hash,
    );

    let mut config = RpcSendTransactionConfig::default();
    config.skip_preflight = false;
    let r = ob_client.rpc_client.send_transaction_with_config(&txn, config);
    println!("got results: {:?}", r);

    Ok(())
}
