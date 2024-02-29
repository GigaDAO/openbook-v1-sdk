


pub fn _initialize_new_oos_account() {
    // let (orders_key, create_oos_account_ix) = create_dex_account(&rpc_client, &program_id_binding, &keypair.pubkey(), size_of::<openbook_dex::state::OpenOrders>())?;
    // let init_ix = openbook_dex::instruction::init_open_orders(
    //     &program_id_binding,
    //     &orders_key.pubkey(),
    //     &keypair.pubkey(),
    //     &market_account_binding,
    //     None,
    // )?;
    // println!("got oos account: {:?}", orders_key.pubkey());
    //
    // let mut instructions = Vec::new();
    // instructions.push(create_oos_account_ix);
    // instructions.push(init_ix);
    //
    // let recent_hash = rpc_client.get_latest_blockhash()?;
    // let txn = Transaction::new_signed_with_payer(
    //     &instructions,
    //     Some(&keypair.pubkey()),
    //     &[&orders_key, &keypair],
    //     recent_hash,
    // );
    //
    // let r = rpc_client.send_transaction(&txn);
    // println!("got results: {:?}", r);
}
