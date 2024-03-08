use openbook_dex::matching::Side;
use solana_program::instruction::Instruction;
use solana_rpc_client_api::config::RpcSendTransactionConfig;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::Transaction;
use crate::ObClient;

pub async fn cancel_all_limit_orders(ob_client: &mut ObClient, execute: bool) -> anyhow::Result<Option<Vec<Instruction>>> {

    let mut ixs = Vec::new();

    for oid in &ob_client.oo_state.open_bids {
        let ix = openbook_dex::instruction::cancel_order(
            &ob_client.program_id,
            &ob_client.market_account,
            &ob_client.oo_state.bids_address,
            &ob_client.oo_state.asks_address,
            &ob_client.open_orders_account,
            &ob_client.keypair.pubkey(),
            &ob_client.event_queue,
            Side::Bid,
            *oid,
        )?;
        ixs.push(ix);
    }

    for oid in &ob_client.oo_state.open_asks {
        let ix = openbook_dex::instruction::cancel_order(
            &ob_client.program_id,
            &ob_client.market_account,
            &ob_client.oo_state.bids_address,
            &ob_client.oo_state.asks_address,
            &ob_client.open_orders_account,
            &ob_client.keypair.pubkey(),
            &ob_client.event_queue,
            Side::Ask,
            *oid,
        )?;
        ixs.push(ix);
    }

    if ixs.len() == 0 {
        return Ok(None);
    }

    if !execute {
        return Ok(Some(ixs));
    }

    let recent_hash = ob_client.rpc_client.get_latest_blockhash().await?;
    let txn = Transaction::new_signed_with_payer(
        &ixs,
        Some(&ob_client.keypair.pubkey()),
        &[&ob_client.keypair],
        recent_hash,
    );

    let mut config = RpcSendTransactionConfig::default();
    config.skip_preflight = false;
    let r = ob_client.rpc_client.send_transaction_with_config(&txn, config).await;
    tracing::debug!("got results: {:?}", r);

    Ok(None)
}
