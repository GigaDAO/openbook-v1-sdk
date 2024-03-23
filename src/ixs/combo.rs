use openbook_dex::matching::Side;
use solana_rpc_client_api::config::RpcSendTransactionConfig;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::Transaction;
use crate::ixs::cancel_limit_order::cancel_limit_order;
use crate::ixs::place_limit_order::place_limit_order;
use crate::ixs::settle_balance::settle_balance;
use crate::ObClient;

pub enum ComboVariants {
    Place,
    SettlePlace,
    CancelSettlePlace,
}

pub async fn combo_cancel_settle_place(
    mut ob_client: &mut ObClient,
    side: Side,
    target_size_usdc: f64,
    price_jlp_usdc: f64,
    oids: Vec<u128>,
    combo: ComboVariants,
) -> anyhow::Result<()> {

    // add priority and budget fee
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

    match combo {
        ComboVariants::Place => {
            let ix = place_limit_order(&mut ob_client, target_size_usdc, side, price_jlp_usdc).await?;
            instructions.push(ix);
        }
        ComboVariants::SettlePlace => {
            let ix = settle_balance(&mut ob_client).await?;
            instructions.push(ix);
            let ix = place_limit_order(&mut ob_client, target_size_usdc, side, price_jlp_usdc).await?;
            instructions.push(ix);
        }
        ComboVariants::CancelSettlePlace => {
            for oid in oids {
                let ix = cancel_limit_order(&mut ob_client, oid, side).await?;
                instructions.push(ix);
            }
            let ix = settle_balance(&mut ob_client).await?;
            instructions.push(ix);
            let ix = place_limit_order(&mut ob_client, target_size_usdc, side, price_jlp_usdc).await?;
            instructions.push(ix);
        }
    }

    // execute
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
