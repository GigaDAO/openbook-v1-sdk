// // TODO extract
// if cancel_order {
//     let ix = openbook_dex::instruction::cancel_order(
//         &program_id_binding,
//         &market_account_binding,
//         &oo_state.bids_address,
//         &oo_state.asks_address,
//         &orders_key,
//         &keypair.pubkey(),
//         &event_queue,
//         Side::Bid,
//         order_id_0,
//     )?;
//     // place order
//     let mut instructions = Vec::new();
//     instructions.push(ix);
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
