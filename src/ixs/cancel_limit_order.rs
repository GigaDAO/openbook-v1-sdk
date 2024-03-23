use openbook_dex::matching::Side;
use solana_program::instruction::Instruction;
use solana_rpc_client_api::config::RpcSendTransactionConfig;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::Transaction;
use crate::ObClient;

pub async fn cancel_limit_order(ob_client: &mut ObClient, oid: u128, side: Side) -> anyhow::Result<Instruction> {

    let ix = openbook_dex::instruction::cancel_order(
        &ob_client.program_id,
        &ob_client.market_account,
        &ob_client.bids_address,
        &ob_client.asks_address,
        &ob_client.open_orders_account,
        &ob_client.keypair.pubkey(),
        &ob_client.event_queue,
        side,
        oid,
    )?;

    Ok(ix)
}
