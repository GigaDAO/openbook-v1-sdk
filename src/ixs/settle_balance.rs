use solana_rpc_client_api::config::RpcSendTransactionConfig;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::Transaction;
use crate::ObClient;

pub fn settle_balance(ob_client: &mut ObClient) -> anyhow::Result<()> {

    let ix = openbook_dex::instruction::settle_funds(
        &ob_client.program_id,
        &ob_client.market_account,
        &anchor_spl::token::ID,
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
    let mut instructions = Vec::new();
    instructions.push(ix);

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
