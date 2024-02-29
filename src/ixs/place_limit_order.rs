// TODO extract the order placement and do math in there...
// let target_size_usdc = 1.0;
// let target_usdc_lots_w_fee = (target_size_usdc * 1e6 * 1.1) as u64;
// let price = oo_state.max_bid as f64 / 1e3 - 30.;
// let new_bid = (price * 1e3) as u64;
// let target_amount_wsol = target_size_usdc / price;
// let target_wsol_lots = (target_amount_wsol * 1e3) as u64;
//
// let limit_price = NonZeroU64::new(new_bid).unwrap();
// let max_coin_qty = NonZeroU64::new(target_wsol_lots).unwrap(); // max wsol lots
// let max_native_pc_qty_including_fees = NonZeroU64::new(target_usdc_lots_w_fee).unwrap(); // max usdc lots + fees
//
// //
//
//
// // TODO extract
// if place_limit_bid {
//     let place_order_ix = openbook_dex::instruction::new_order(
//         &market_account_binding,
//         &orders_key,
//         &request_queue,
//         &event_queue,
//         &oo_state.bids_address,
//         &oo_state.asks_address,
//         &usdc_ata,
//         &keypair.pubkey(),
//         &coin_vault,
//         &pc_vault,
//         &anchor_spl::token::ID,
//         &solana_program::sysvar::rent::ID,
//         None,
//         &program_id_binding,
//         Side::Bid,
//         limit_price,
//         max_coin_qty,
//         OrderType::PostOnly,
//         random::<u64>(),
//         SelfTradeBehavior::AbortTransaction,
//         u16::MAX,
//         max_native_pc_qty_including_fees,
//         (get_unix_secs() + 30) as i64,
//     )?;
//
//     // place order
//     let mut instructions = Vec::new();
//     instructions.push(place_order_ix);
//
//     let recent_hash = rpc_client.get_latest_blockhash()?;
//     let txn = Transaction::new_signed_with_payer(
//         &instructions,
//         Some(&keypair.pubkey()),
//         &[&keypair],
//         recent_hash,
//     );
//
//     let mut config = RpcSendTransactionConfig::default();
//     config.skip_preflight = true;
//     let r = rpc_client.send_transaction_with_config(&txn, config);
//     println!("got results: {:?}", r);
// }
