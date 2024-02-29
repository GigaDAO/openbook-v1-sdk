use solana_program::pubkey::Pubkey;
use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use crate::utils::create_dex_account;

pub fn initialize_new_oos_account(
    rpc_client: &mut RpcClient,
    program_id: Pubkey,
    keypair: Keypair,
    market_account: Pubkey,
) -> anyhow::Result<()>{

    let (orders_key, create_oos_account_ix) = create_dex_account(
        &rpc_client,
        &program_id,
        &keypair.pubkey(),
        std::mem::size_of::<openbook_dex::state::OpenOrders>())?;
    let init_ix = openbook_dex::instruction::init_open_orders(
        &program_id,
        &orders_key.pubkey(),
        &keypair.pubkey(),
        &market_account,
        None,
    )?;
    println!("got oos account: {:?}", orders_key.pubkey());

    let mut instructions = Vec::new();
    instructions.push(create_oos_account_ix);
    instructions.push(init_ix);

    let recent_hash = rpc_client.get_latest_blockhash()?;
    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&keypair.pubkey()),
        &[&orders_key, &keypair],
        recent_hash,
    );

    let r = rpc_client.send_transaction(&txn);
    println!("got results: {:?}", r);

    Ok(())
}
