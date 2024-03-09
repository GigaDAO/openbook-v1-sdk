use openbook_dex::matching::Side;
use solana_rpc_client_api::config::RpcSendTransactionConfig;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::Transaction;
use crate::ixs::cancel_limit_order::cancel_all_limit_orders;
use crate::ixs::place_limit_order::place_limit_order;
use crate::ixs::settle_balance::settle_balance;
use crate::ObClient;

pub async fn combo_cancel_settle_place(
    mut ob_client: &mut ObClient,
    target_size_usdc_ask: f64,
    target_size_usdc_bid: f64,
    bid_price_jlp_usdc: f64,
    ask_price_jlp_usdc: f64
) -> anyhow::Result<()> {

    let mut instructions = Vec::new();
    let r = ob_client.rpc_client.get_recent_prioritization_fees(&[]).await.unwrap();
    let mut max_fee = 1;
    for f in r {
        if f.prioritization_fee > max_fee {
            max_fee = f.prioritization_fee;
        }
    }

    let budget_ix = ComputeBudgetInstruction::set_compute_unit_limit(1_000_000);
    let fee_ix = ComputeBudgetInstruction::set_compute_unit_price(max_fee);
    instructions.push(budget_ix);
    instructions.push(fee_ix);

    let ixs = cancel_all_limit_orders(&mut ob_client, false).await?;
    if let Some(ixs) = ixs {
        instructions.extend(ixs);
    }
    let ixs = settle_balance(&mut ob_client, false).await?.unwrap();
    instructions.extend(ixs);
    let ixs = place_limit_order(&mut ob_client, target_size_usdc_bid, Side::Bid, 0., false, bid_price_jlp_usdc).await?.unwrap();
    instructions.extend(ixs);
    let ixs = place_limit_order(&mut ob_client, target_size_usdc_ask, Side::Ask, 0., false, ask_price_jlp_usdc).await?.unwrap();
    instructions.extend(ixs);

    let recent_hash = ob_client.rpc_client.get_latest_blockhash_with_commitment(CommitmentConfig::confirmed()).await?;
    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&ob_client.keypair.pubkey()),
        &[&ob_client.keypair],
        recent_hash.0,
    );

    let mut config = RpcSendTransactionConfig::default();
    config.skip_preflight = true;
    config.preflight_commitment = Some(CommitmentLevel::Confirmed);
    let kp_str = ob_client.keypair.pubkey().to_string().clone();
    match ob_client.rpc_client.send_transaction_with_config(&txn, config).await{
        Ok(_) => {}
        Err(err) => {tracing::error!("err combo'ing: {err}, {}", kp_str);}
    }

    Ok(())
}
pub async fn combo_cancel_settle_place_bid(
    mut ob_client: &mut ObClient,
    target_size_usdc_bid: f64,
    bid_price_jlp_usdc: f64,
) -> anyhow::Result<()> {

    let mut instructions = Vec::new();


    let r = ob_client.rpc_client.get_recent_prioritization_fees(&[]).await.unwrap();
    let mut max_fee = 1;
    for f in r {
        if f.prioritization_fee > max_fee {
            max_fee = f.prioritization_fee;
        }
    }

    let budget_ix = ComputeBudgetInstruction::set_compute_unit_limit(800_000);
    let fee_ix = ComputeBudgetInstruction::set_compute_unit_price(max_fee);
    instructions.push(budget_ix);
    instructions.push(fee_ix);

    let ixs = cancel_all_limit_orders(&mut ob_client, false).await?;
    if let Some(ixs) = ixs {
        instructions.extend(ixs);
    }
    let ixs = settle_balance(&mut ob_client, false).await?.unwrap();
    instructions.extend(ixs);

    let ixs = place_limit_order(&mut ob_client, target_size_usdc_bid, Side::Bid, 0., false, bid_price_jlp_usdc).await?;
    if let Some(ixs) = ixs {
        instructions.extend(ixs);
    }

    let recent_hash = ob_client.rpc_client.get_latest_blockhash_with_commitment(CommitmentConfig::confirmed()).await?;
    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&ob_client.keypair.pubkey()),
        &[&ob_client.keypair],
        recent_hash.0,
    );
    let mut config = RpcSendTransactionConfig::default();
    config.skip_preflight = true;
    config.preflight_commitment = Some(CommitmentLevel::Confirmed);
    let kp_str = ob_client.keypair.pubkey().to_string().clone();
    match ob_client.rpc_client.send_transaction_with_config(&txn, config).await{
        Ok(_) => {}
        Err(err) => {tracing::error!("err bidding: {err}, {}", kp_str);
            let e = err.get_transaction_error();
            if let Some(e) = e {
                tracing::error!("got tx err: {e}"); }
        }
    }

    Ok(())
}

// TODO consolidate all these helpers...
pub async fn combo_cancel_settle_place_ask(
    mut ob_client: &mut ObClient,
    target_size_usdc_ask: f64,
    ask_price_jlp_usdc: f64,
) -> anyhow::Result<()> {

    let mut instructions = Vec::new();


    let r = ob_client.rpc_client.get_recent_prioritization_fees(&[]).await.unwrap();
    let mut max_fee = 1;
    for f in r {
        if f.prioritization_fee > max_fee {
            max_fee = f.prioritization_fee;
        }
    }

    let budget_ix = ComputeBudgetInstruction::set_compute_unit_limit(800_000);
    let fee_ix = ComputeBudgetInstruction::set_compute_unit_price(max_fee);
    instructions.push(budget_ix);
    instructions.push(fee_ix);

    let ixs = cancel_all_limit_orders(&mut ob_client, false).await?;
    if let Some(ixs) = ixs {
        instructions.extend(ixs);
    }

    let ixs = settle_balance(&mut ob_client, false).await?.unwrap();
    instructions.extend(ixs);

    let ixs = place_limit_order(&mut ob_client, target_size_usdc_ask, Side::Ask, 0., false, ask_price_jlp_usdc).await?;
    if let Some(ixs) = ixs {
        instructions.extend(ixs);
    }

    let recent_hash = ob_client.rpc_client.get_latest_blockhash().await?;
    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&ob_client.keypair.pubkey()),
        &[&ob_client.keypair],
        recent_hash,
    );

    let mut config = RpcSendTransactionConfig::default();
    config.skip_preflight = true;
    let kp_str = ob_client.keypair.pubkey().to_string().clone();
    match ob_client.rpc_client.send_transaction_with_config(&txn, config).await{
        Ok(_) => {}
        Err(err) => {tracing::error!("err asking: {err}, {}", kp_str);}
    }
    // tracing::info!("got results: {:?}", r);

    Ok(())
}
pub async fn combo_cancel_settle(
    mut ob_client: &mut ObClient,
) -> anyhow::Result<()> {
    let mut instructions = Vec::new();
    let r = ob_client.rpc_client.get_recent_prioritization_fees(&[]).await.unwrap();
    let mut max_fee = 1;
    for f in r {
        if f.prioritization_fee > max_fee {
            max_fee = f.prioritization_fee;
        }
    }
    let budget_ix = ComputeBudgetInstruction::set_compute_unit_limit(800_000);
    let fee_ix = ComputeBudgetInstruction::set_compute_unit_price(max_fee);
    instructions.push(budget_ix);
    instructions.push(fee_ix);

    let ixs = cancel_all_limit_orders(&mut ob_client, false).await?;
    if let Some(ixs) = ixs {
        instructions.extend(ixs);
    }
    let ixs = settle_balance(&mut ob_client, false).await?.unwrap();
    instructions.extend(ixs);

    // TODO add commitment to this
    let recent_hash = ob_client.rpc_client.get_latest_blockhash_with_commitment(CommitmentConfig::confirmed()).await?;
    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&ob_client.keypair.pubkey()),
        &[&ob_client.keypair],
        recent_hash.0,
    );

    // TODO add commitment level
    let mut config = RpcSendTransactionConfig::default();
    config.skip_preflight = true;
    config.preflight_commitment = Some(CommitmentLevel::Confirmed);
    let kp_str = ob_client.keypair.pubkey().to_string().clone();

    // TODO should definitely add retry logic
    match ob_client.rpc_client.send_transaction_with_config(&txn, config).await {
        Ok(v) => {println!("{:?}", v)}
        Err(e) => {println!("err canceling: {e}\npk: {}", kp_str);}
    }
    Ok(())
}
