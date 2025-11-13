use anyhow::{Context, Result};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use subxt::ext::sp_core::{crypto::Ss58Codec, sr25519::Pair, Pair as PairTrait};
use subxt::tx::PairSigner;
use subxt::{OnlineClient, PolkadotConfig};
use tokio::sync::{Mutex, Semaphore};
use tracing::{info, warn};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Transaction {
    tx_id: String,
    tx_type: String,
    trader: String,
    params: serde_json::Value,
    timestamp: u64,
    nonce: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct Block {
    block_number: u64,
    timestamp: u64,
    transactions: Vec<Transaction>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PlaceOrderParams {
    side: String,
    price: f64,
    quantity: f64,
}

#[derive(Debug, Deserialize, Serialize)]
struct CancelOrderParams {
    order_id: String,
}

// Generate metadata at compile time
#[subxt::subxt(runtime_metadata_path = "../metadata.scale")]
pub mod polkadot {}

// Import the generated types for convenience
use polkadot::runtime_types::pallet_orderbook::types::{OrderSide, OrderType};

struct TradeBot {
    client: OnlineClient<PolkadotConfig>,
    accounts: Vec<(String, Pair)>, // (address, keypair)
    account_locks: HashMap<String, Arc<Mutex<()>>>, // Per-account locks
    worker_pool: Arc<Semaphore>,   // Limits concurrent workers
}

impl TradeBot {
    async fn new(node_url: &str, num_accounts: usize, pool_size: usize) -> Result<Self> {
        let client = OnlineClient::<PolkadotConfig>::from_url(node_url)
            .await
            .context("Failed to connect to node")?;

        info!("✅ Connected to chain: {:?}", client.runtime_version());

        // Generate accounts using development keypairs
        let accounts = Self::generate_accounts(num_accounts)?;

        info!("Generated {} trading accounts:", accounts.len());
        for (addr, _) in &accounts {
            info!("  - {}", addr);
        }

        // Create a lock for each account upfront
        let mut account_locks = HashMap::new();
        for (addr, _) in &accounts {
            account_locks.insert(addr.clone(), Arc::new(Mutex::new(())));
        }

        Ok(Self {
            client,
            accounts,
            account_locks,
            worker_pool: Arc::new(Semaphore::new(pool_size)),
        })
    }

    fn generate_accounts(num: usize) -> Result<Vec<(String, Pair)>> {
        // Dev account URIs (well-known substrate test accounts)
        let dev_uris = [
            "//Alice",
            "//Bob",
            "//Charlie",
            "//Dave",
            "//Eve",
            "//Ferdie",
        ];

        let mut accounts = Vec::new();
        for i in 0..num {
            let uri = dev_uris[i % dev_uris.len()];
            let pair = Pair::from_string(uri, None)
                .map_err(|e| anyhow::anyhow!("Failed to create pair from URI {}: {:?}", uri, e))?;

            // Get the public key and convert to proper SS58 address
            let public = pair.public();
            let address = public.to_ss58check();

            accounts.push((address, pair));
        }

        Ok(accounts)
    }

    fn map_trader_to_account(&self, trader: &str) -> &(String, Pair) {
        // Hash the trader string to deterministically map to an account
        let hash: u64 = trader
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        let index = (hash as usize) % self.accounts.len();
        &self.accounts[index]
    }

    async fn place_order(&self, trader: &str, side: &str, price: f64, quantity: f64) -> Result<()> {
        let (address, pair) = self.map_trader_to_account(trader);

        // Get the lock for this specific account (already created in new())
        let account_lock = self
            .account_locks
            .get(address)
            .expect("Account lock should exist")
            .clone();

        // Acquire the lock for this account to ensure sequential transactions
        let _guard = account_lock.lock().await;

        // Convert f64 price and quantity to u128 (assuming 6 decimal places)
        let price_u128 = (price * 1_000_000.0) as u128;
        let quantity_u128 = (quantity * 1_000_000.0) as u128;

        // Determine order side
        let order_side = if side.to_lowercase() == "bid" || side.to_lowercase() == "buy" {
            OrderSide::Buy
        } else {
            OrderSide::Sell
        };

        // Use Limit order type for all orders from the synthetic data
        let order_type = OrderType::Limit;

        // Create a PairSigner for subxt
        let signer = PairSigner::new(pair.clone());

        // Build the extrinsic with correct parameter order: side, price, quantity, order_type
        let tx = polkadot::tx().orderbook().place_order(
            order_side,
            price_u128,
            quantity_u128,
            order_type,
        );

        // Wait for confirmation to avoid nonce issues
        match self
            .client
            .tx()
            .sign_and_submit_then_watch_default(&tx, &signer)
            .await
        {
            Ok(progress) => {
                match progress.wait_for_finalized_success().await {
                    Ok(_) => {
                        info!(
                            "✅ Order placed: {} {} @ {} (qty: {})",
                            address, side, price, quantity
                        );
                        Ok(())
                    }
                    Err(e) => {
                        warn!(
                            "⚠️  Order failed (finalization): {} {} @ {} - {}",
                            address, side, price, e
                        );
                        // Don't return error, just log and continue
                        Ok(())
                    }
                }
            }
            Err(e) => {
                warn!(
                    "⚠️  Order failed (submission): {} {} @ {} - {}",
                    address, side, price, e
                );
                // Don't return error, just log and continue
                Ok(())
            }
        }
    }

    async fn cancel_order(
        &self,
        trader: &str,
        _order_id: &str, // We don't have real order IDs from the chain
    ) -> Result<()> {
        let (address, _pair) = self.map_trader_to_account(trader);

        // For now, we'll skip cancel operations since we don't have real order IDs
        // from the chain matching the synthetic data
        // If we implement this in the future, it should use sign_and_submit_then_watch_default
        // and wait_for_finalized_success to avoid nonce issues
        info!("⏭️  Skipping cancel order for {}", address);
        Ok(())
    }

    async fn fund_accounts(&self) -> Result<()> {
        info!("💰 Funding accounts with ETH (asset 0) and USDC (asset 1)...");

        // Fund amount: 1 trillion with 6 decimals = 1_000_000_000_000 * 1_000_000
        let fund_amount: u128 = 1_000_000_000_000_000_000;

        // Fund accounts sequentially to avoid nonce conflicts
        for (address, pair) in &self.accounts {
            let signer = PairSigner::new(pair.clone());

            // Fund with ETH (asset_id = 0)
            let deposit_eth = polkadot::tx().assets().deposit(0, fund_amount);

            match self
                .client
                .tx()
                .sign_and_submit_then_watch_default(&deposit_eth, &signer)
                .await
            {
                Ok(progress) => match progress.wait_for_finalized_success().await {
                    Ok(_) => {
                        info!("✅ Funded {} with {} ETH", address, fund_amount / 1_000_000);
                    }
                    Err(e) => {
                        warn!(
                            "⚠️  ETH deposit failed (finalization) for {}: {}",
                            address, e
                        );
                        return Err(anyhow::anyhow!("ETH deposit failed for {}", address));
                    }
                },
                Err(e) => {
                    warn!("⚠️  Failed to submit ETH deposit for {}: {}", address, e);
                    return Err(anyhow::anyhow!(
                        "Failed to submit ETH deposit for {}",
                        address
                    ));
                }
            }

            // Fund with USDC (asset_id = 1)
            let deposit_usdc = polkadot::tx().assets().deposit(1, fund_amount);

            match self
                .client
                .tx()
                .sign_and_submit_then_watch_default(&deposit_usdc, &signer)
                .await
            {
                Ok(progress) => match progress.wait_for_finalized_success().await {
                    Ok(_) => {
                        info!(
                            "✅ Funded {} with {} USDC",
                            address,
                            fund_amount / 1_000_000
                        );
                    }
                    Err(e) => {
                        warn!(
                            "⚠️  USDC deposit failed (finalization) for {}: {}",
                            address, e
                        );
                        return Err(anyhow::anyhow!("USDC deposit failed for {}", address));
                    }
                },
                Err(e) => {
                    warn!("⚠️  Failed to submit USDC deposit for {}: {}", address, e);
                    return Err(anyhow::anyhow!(
                        "Failed to submit USDC deposit for {}",
                        address
                    ));
                }
            }
        }

        info!("✅ Account funding complete");
        Ok(())
    }

    async fn process_transaction(&self, tx: &Transaction) -> Result<()> {
        match tx.tx_type.as_str() {
            "place_order" => {
                let params: PlaceOrderParams = serde_json::from_value(tx.params.clone())
                    .context("Failed to parse place_order params")?;

                self.place_order(&tx.trader, &params.side, params.price, params.quantity)
                    .await?;
            }
            "cancel_order" => {
                let params: CancelOrderParams = serde_json::from_value(tx.params.clone())
                    .context("Failed to parse cancel_order params")?;

                self.cancel_order(&tx.trader, &params.order_id).await?;
            }
            _ => {
                warn!("Unknown transaction type: {}", tx.tx_type);
            }
        }
        Ok(())
    }

    async fn replay_transactions(self: Arc<Self>, blocks: Vec<Block>) -> Result<()> {
        // Flatten all transactions from all blocks into a single list
        let all_txs: Vec<Transaction> = blocks
            .into_iter()
            .flat_map(|block| block.transactions)
            .collect();

        info!(
            "🚀 Starting transaction replay with {} transactions",
            all_txs.len()
        );
        info!(
            "👷 Worker pool size: {}",
            self.worker_pool.available_permits()
        );

        let mut handles = Vec::new();

        // Spawn a worker for each transaction
        for tx in all_txs {
            // Acquire a permit from the worker pool (blocks if pool is saturated)
            let permit = self.worker_pool.clone().acquire_owned().await.unwrap();
            let bot = self.clone();

            // Spawn worker task
            let handle = tokio::spawn(async move {
                let result = bot.process_transaction(&tx).await;

                // Log result
                match &result {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("Failed to process tx {}: {}", tx.tx_id, e);
                    }
                }

                // Permit is automatically released when dropped
                drop(permit);
                result
            });

            handles.push(handle);
        }

        // Wait for all workers to complete
        info!("⏳ Waiting for all workers to complete...");
        let results = futures::future::join_all(handles).await;

        // Count results
        let mut submitted = 0;
        let mut failed = 0;
        for result in results {
            match result {
                Ok(Ok(_)) => submitted += 1,
                _ => failed += 1,
            }
        }

        info!(
            "✅ Completed transaction replay: {} submitted, {} failed",
            submitted, failed
        );
        Ok(())
    }
}

fn load_blocks(file_path: &str) -> Result<Vec<Block>> {
    info!("📂 Loading blocks from: {}", file_path);

    let file =
        File::open(file_path).with_context(|| format!("Failed to open file: {}", file_path))?;

    let reader = BufReader::new(file);
    let mut blocks = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("Failed to read line {}", line_num + 1))?;

        if line.trim().is_empty() {
            continue;
        }

        let block: Block = serde_json::from_str(&line)
            .with_context(|| format!("Failed to parse block at line {}", line_num + 1))?;

        blocks.push(block);
    }

    info!("✅ Loaded {} blocks", blocks.len());
    Ok(blocks)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    info!("🤖 Trade Bot Starting...");

    // Configuration from environment
    let node_url = env::var("NODE_WS_URL").unwrap_or_else(|_| "ws://127.0.0.1:9944".to_string());
    let num_accounts = env::var("NUM_ACCOUNTS")
        .unwrap_or_else(|_| "6".to_string())
        .parse::<usize>()
        .context("NUM_ACCOUNTS must be a valid number")?;

    let worker_pool_size = env::var("WORKER_POOL_SIZE")
        .unwrap_or_else(|_| "10".to_string())
        .parse::<usize>()
        .context("WORKER_POOL_SIZE must be a valid number")?;

    let blocks_file = env::var("BLOCKS_FILE")
        .unwrap_or_else(|_| "ETHUSDC_2025-11-12T22-08-37-339Z_synthetic_blocks.jsonl".to_string());

    info!("Configuration:");
    info!("  Node URL: {}", node_url);
    info!("  Num Accounts: {}", num_accounts);
    info!("  Worker Pool Size: {}", worker_pool_size);
    info!("  Blocks File: {}", blocks_file);

    // Initialize trade bot
    let bot = TradeBot::new(&node_url, num_accounts, worker_pool_size).await?;

    // Load blocks from file
    let blocks = load_blocks(&blocks_file)?;

    // Fund accounts if not skipped
    if env::var("SKIP_FUNDING").unwrap_or_else(|_| "0".to_string()) != "1" {
        bot.fund_accounts().await?;
    } else {
        info!("⏭️  Skipping account funding (SKIP_FUNDING=1)");
    }

    // Wrap bot in Arc for shared ownership across workers
    let bot = Arc::new(bot);

    // Replay all transactions with worker pool
    bot.replay_transactions(blocks).await?;

    info!("🎉 Trade bot completed successfully!");

    Ok(())
}
