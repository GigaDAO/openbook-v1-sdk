use solana_program::instruction::Instruction;
use solana_rpc_client_api::config::RpcSendTransactionConfig;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::Transaction;
use crate::ObClient;

pub async fn settle_balance(ob_client: &mut ObClient) -> anyhow::Result<Instruction> {

    let ix = openbook_dex::instruction::settle_funds(
        &ob_client.program_id,
        &ob_client.market_account,
        &spl_token::ID,
        &ob_client.open_orders_account,
        &ob_client.keypair.pubkey(),
        &ob_client.coin_vault,
        &ob_client.base_ata,
        &ob_client.pc_vault,
        &ob_client.quote_ata,
        None,
        &ob_client.vault_signer_key,
    )?;

    // place order
    // let mut instructions = Vec::new();
    // instructions.push(ix);

    Ok(ix)

    // let recent_hash = ob_client.rpc_client.get_latest_blockhash().await?;
    // let txn = Transaction::new_signed_with_payer(
    //     &instructions,
    //     Some(&ob_client.keypair.pubkey()),
    //     &[&ob_client.keypair],
    //     recent_hash,
    // );
    //
    // let mut config = RpcSendTransactionConfig::default();
    // config.skip_preflight = false;
    // let r = ob_client.rpc_client.send_transaction_with_config(&txn, config).await;
    // tracing::debug!("got results: {:?}", r);
    //
    // Ok(None)
}
