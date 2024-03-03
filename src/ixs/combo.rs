use openbook_dex::matching::Side;
use solana_rpc_client_api::config::RpcSendTransactionConfig;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::Transaction;
use crate::ixs::cancel_limit_order::cancel_all_limit_orders;
use crate::ixs::place_limit_order::place_limit_order;
use crate::ixs::settle_balance::settle_balance;
use crate::ObClient;

pub fn combo_cancel_settle_place(
    mut ob_client: &mut ObClient,
    target_size_usdc: f64,
    bid_price_jlp_usdc: f64,
    ask_price_jlp_usdc: f64
) -> anyhow::Result<()> {

    let mut instructions = Vec::new();
    let ixs = cancel_all_limit_orders(&mut ob_client, false)?;
    if let Some(ixs) = ixs {
        instructions.extend(ixs);
    }
    let ixs = settle_balance(&mut ob_client, false)?.unwrap();
    instructions.extend(ixs);
    let ixs = place_limit_order(&mut ob_client, target_size_usdc, Side::Bid, 0., false, bid_price_jlp_usdc)?.unwrap();
    instructions.extend(ixs);
    // TODO put this back
    // let ixs = place_limit_order(&mut ob_client, target_size_usdc, Side::Ask, 0., false, ask_price_jlp_usdc)?.unwrap();
    // instructions.extend(ixs);

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
