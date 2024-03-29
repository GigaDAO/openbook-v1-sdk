/// The entry point for the OpenBook CLI application.
///
/// # Returns
///
/// Returns `Ok(())` on success or an error if an error occurs.|
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "cli")]
    {
        use clap::Parser;
        use openbook::cli::{Cli, Commands};
        use openbook::commitment_config::CommitmentConfig;
        use openbook::market::Market;
        use openbook::matching::Side;
        use openbook::rpc_client::RpcClient;
        use openbook::utils::read_keypair;

        let rpc_url = std::env::var("RPC_URL").expect("RPC_URL is not set");
        let key_path = std::env::var("KEY_PATH").expect("KEY_PATH is not set");

        let args = Cli::parse();

        let commitment_config = CommitmentConfig::confirmed();
        let rpc_client = RpcClient::new_with_commitment(rpc_url, commitment_config);

        let keypair = read_keypair(&key_path);

        let mut market = Market::new(rpc_client, 3, "usdc", keypair).await;

        match args.command {
            Some(Commands::Info(_)) => {
                println!("[*] Market Info: {:?}", market);
            }
            Some(Commands::Place(arg)) => {
                let side = match arg.side.as_str() {
                    "bid" => Side::Bid,
                    "ask" => Side::Ask,
                    _ => Side::Bid,
                };

                let result = market
                    .place_limit_order(
                        arg.target_amount_quote,
                        side,
                        arg.best_offset_usdc,
                        arg.execute,
                        arg.price_target,
                    )
                    .await?;
                println!(
                    "[*] Transaction successful, signature: {:?}",
                    result.unwrap()
                );
            }
            Some(Commands::Cancel(arg)) => {
                let c = market.cancel_orders(arg.execute).await?;
                println!("[*] Transaction successful, signature: {:?}", c);
            }
            Some(Commands::Settle(arg)) => {
                let result = market.settle_balance(arg.execute).await?;
                println!(
                    "[*] Transaction successful, signature: {:?}",
                    result.unwrap()
                );
            }
            Some(Commands::Match(arg)) => {
                let m = market.make_match_orders_transaction(arg.limit).await?;
                println!("[*] Transaction successful, signature: {:?}", m);
            }
            Some(Commands::Consume(arg)) => {
                let e = market
                    .make_consume_events_instruction(Vec::new(), arg.limit)
                    .await?;
                println!("[*] Transaction successful, signature: {:?}", e);
            }
            Some(Commands::ConsumePermissioned(arg)) => {
                let p = market
                    .make_consume_events_permissioned_instruction(Vec::new(), arg.limit)
                    .await?;
                println!("[*] Transaction successful, signature: {:?}", p);
            }
            Some(Commands::Load(_arg)) => {
                let l = market.load_orders_for_owner().await?;
                println!("[*] Found Program Accounts: {:?}", l);
            }
            Some(Commands::Find(_arg)) => {
                // Todo: Handle Find Open Accounts command
                println!("[*] Find Open Accounts Results: Placeholder");
            }
            None => println!(
                "\x1b[1;91m{}\x1b[0m",
                "Unknown command. Use '--help' for usage instructions."
            ),
        };
    }
    Ok(())
}
