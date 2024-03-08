use solana_program::pubkey::Pubkey;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use crate::utils::create_dex_account;
use solana_sdk::compute_budget::ComputeBudgetInstruction;

pub async fn initialize_new_oos_account(
    rpc_client: &mut RpcClient,
    program_id: Pubkey,
    keypair: &Keypair,
    market_account: Pubkey,
) -> anyhow::Result<Pubkey>{

    let (orders_key, create_oos_account_ix) = create_dex_account(
        &rpc_client,
        &program_id,
        &keypair.pubkey(),
        std::mem::size_of::<openbook_dex::state::OpenOrders>()).await?;
    let init_ix = openbook_dex::instruction::init_open_orders(
        &program_id,
        &orders_key.pubkey(),
        &keypair.pubkey(),
        &market_account,
        None,
    )?;
    println!("got oos account: {:?}", orders_key.pubkey());

    let mut instructions = Vec::new();
    let r = rpc_client.get_recent_prioritization_fees(&[]).await.unwrap();
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


    instructions.push(create_oos_account_ix);
    instructions.push(init_ix);

    println!("using kp: {}", &keypair.pubkey().to_string());

    let recent_hash = rpc_client.get_latest_blockhash().await?;
    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&keypair.pubkey()),
        &[&orders_key, &keypair],
        recent_hash,
    );

    let r = rpc_client.send_transaction(&txn).await?;
    println!("got results: {:?}", r);

    Ok(orders_key.pubkey())
}
