use anyhow::{Context, Result};
use dotenvy::dotenv;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use subxt::{OnlineClient, PolkadotConfig};
use subxt_signer::sr25519::Keypair;
use subxt_signer::sr25519::dev::alice;
use tokio::sync::Mutex;
use tracing::{info, warn};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OrderRecord {
    timestamp: String,
    side: String,
    price: f64,
    size: f64,
    sequence: u64,
}

// Generate metadata at compile time
#[subxt::subxt(runtime_metadata_path = "../metadata.scale")]
pub mod polkadot {}

// Import the generated types for convenience
use polkadot::runtime_types::pallet_orderbook::types::{OrderSide, OrderType};

struct TradeBot {
    client: OnlineClient<PolkadotConfig>,
    accounts: Vec<(String, Keypair)>, // (address, keypair)
    account_locks: HashMap<String, Arc<Mutex<()>>>, // Per-account locks
}

impl TradeBot {
    async fn new(node_url: &str, num_accounts: usize) -> Result<Self> {
        let client = OnlineClient::<PolkadotConfig>::from_url(node_url)
            .await
            .context("Failed to connect to node")?;

        info!("✅ Connected to chain: {:?}", client.runtime_version());

        // Generate accounts using development keypairs
        let accounts = Self::generate_accounts(num_accounts)?;

        info!("Generated {} trading accounts:", accounts.len());

        // Create a lock for each account upfront
        let mut account_locks = HashMap::new();
        for (addr, _) in &accounts {
            account_locks.insert(addr.clone(), Arc::new(Mutex::new(())));
        }

        Ok(Self {
            client,
            accounts,
            account_locks,
        })
    }

    fn generate_accounts(num: usize) -> Result<Vec<(String, Keypair)>> {
        let mut accounts = Vec::new();

        let mut rng = rand::rng();
        let mut seed = [0u8; 32];
        for _ in 0..num {
            rng.fill(&mut seed);

            // Generate a completely random keypair
            let pair = Keypair::from_secret_key(seed)?;

            // Get the public key and convert to proper SS58 address
            let public = pair.public_key();
            let address = public.to_account_id().to_string();

            accounts.push((address, pair));
        }

        Ok(accounts)
    }

    fn get_account_for_order(&self, sequence: u64) -> &(String, Keypair) {
        // Map each order to an account based on sequence number
        let index = (sequence as usize) % self.accounts.len();
        &self.accounts[index]
    }

    async fn place_order(
        &self,
        sequence: u64,
        side: &str,
        price: f64,
        quantity: f64,
    ) -> Result<()> {
        let (address, pair) = self.get_account_for_order(sequence);

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

        // Determine order side ("bid" = Buy, "ask" = Sell)
        let order_side = if side.to_lowercase() == "bid" {
            OrderSide::Buy
        } else {
            OrderSide::Sell
        };

        // Use Limit order type for all orders from the synthetic data
        let order_type = OrderType::Limit;

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
            .sign_and_submit_then_watch_default(&tx, pair)
            .await
        {
            Ok(progress) => {
                match progress.wait_for_finalized().await {
                    Ok(_) => {
                        info!(
                            "✅ Order submitted: {} {} @ {} by {}",
                            side, quantity, price, address
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

    async fn fund_accounts(&self) -> Result<()> {
        info!(
            "💰 Funding accounts with native tokens for tx fees, ETH (asset 0) and USDT (asset 1)..."
        );

        // Fund amount: 1 trillion with 6 decimals = 1_000_000_000_000 * 1_000_000
        let fund_amount: u128 = 1_000_000_000_000_000_000;

        // Amount of native tokens for transaction fees (e.g., 1000 units)
        let native_token_amount: u128 = 1_000_000_000_000_000;

        // Get Alice's keypair to transfer native tokens
        let alice_pair = alice();

        // Fund accounts sequentially to avoid nonce conflicts
        for (address, pair) in &self.accounts {
            // First, transfer native tokens from Alice for transaction fees
            let dest = pair.public_key().to_address();

            let transfer_tx = polkadot::tx()
                .balances()
                .transfer_allow_death(dest, native_token_amount);

            match self
                .client
                .tx()
                .sign_and_submit_then_watch_default(&transfer_tx, &alice_pair)
                .await
            {
                Ok(progress) => match progress.wait_for_finalized().await {
                    Ok(_) => {
                        info!(
                            "✅ Transferred {} native tokens to {} for tx fees",
                            native_token_amount, address
                        );
                    }
                    Err(e) => {
                        warn!(
                            "⚠️  Native token transfer failed (finalization) for {}: {}",
                            address, e
                        );
                        return Err(anyhow::anyhow!(
                            "Native token transfer failed for {}",
                            address
                        ));
                    }
                },
                Err(e) => {
                    warn!(
                        "⚠️  Failed to submit native token transfer for {}: {}",
                        address, e
                    );
                    return Err(anyhow::anyhow!(
                        "Failed to submit native token transfer for {}",
                        address
                    ));
                }
            }
            // Fund with ETH (asset_id = 0)
            let deposit_eth = polkadot::tx().assets().deposit(0, fund_amount);

            match self
                .client
                .tx()
                .sign_and_submit_then_watch_default(&deposit_eth, pair)
                .await
            {
                Ok(progress) => match progress.wait_for_finalized().await {
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

            // Fund with USDT (asset_id = 1)
            let deposit_usdt = polkadot::tx().assets().deposit(1, fund_amount);

            match self
                .client
                .tx()
                .sign_and_submit_then_watch_default(&deposit_usdt, pair)
                .await
            {
                Ok(progress) => match progress.wait_for_finalized().await {
                    Ok(_) => {
                        info!(
                            "✅ Funded {} with {} USDT",
                            address,
                            fund_amount / 1_000_000
                        );
                    }
                    Err(e) => {
                        warn!(
                            "⚠️  USDT deposit failed (finalization) for {}: {}",
                            address, e
                        );
                        return Err(anyhow::anyhow!("USDT deposit failed for {}", address));
                    }
                },
                Err(e) => {
                    warn!("⚠️  Failed to submit USDT deposit for {}: {}", address, e);
                    return Err(anyhow::anyhow!(
                        "Failed to submit USDT deposit for {}",
                        address
                    ));
                }
            }
        }

        info!("✅ Account funding complete");
        Ok(())
    }

    async fn process_order(&self, order: &OrderRecord) -> Result<()> {
        self.place_order(order.sequence, &order.side, order.price, order.size)
            .await
    }

    async fn replay_orders(self: Arc<Self>, orders: Vec<OrderRecord>) -> Result<()> {
        info!("🚀 Starting order replay with {} orders", orders.len());

        let mut handles = Vec::new();

        // Spawn a worker for each order
        for order in orders {
            if order.size <= 0.0 {
                warn!(
                    "⚠️  Skipping order with non-positive size: seq {} size {}",
                    order.sequence, order.size
                );
                continue;
            }
            // Acquire a permit from the worker pool (blocks if pool is saturated)
            let bot = self.clone();

            // Spawn worker task
            let handle = tokio::spawn(async move {
                let result = bot.process_order(&order).await;

                // Log result
                match &result {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("Failed to process order seq {}: {}", order.sequence, e);
                    }
                }

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
            "✅ Completed order replay: {} submitted, {} failed",
            submitted, failed
        );
        Ok(())
    }
}

fn load_orders(file_path: &str) -> Result<Vec<OrderRecord>> {
    info!("📂 Loading orders from: {}", file_path);

    let file =
        File::open(file_path).with_context(|| format!("Failed to open file: {}", file_path))?;

    let reader = BufReader::new(file);
    let mut orders = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("Failed to read line {}", line_num + 1))?;

        if line.trim().is_empty() {
            continue;
        }

        let order: OrderRecord = serde_json::from_str(&line)
            .with_context(|| format!("Failed to parse order at line {}", line_num + 1))?;

        orders.push(order);
    }

    info!("✅ Loaded {} orders", orders.len());
    Ok(orders)
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

    let order_data_file = env::var("ORDER_DATA_FILE")?;

    info!("Configuration:");
    info!("  Node URL: {}", node_url);
    info!("  Num Accounts: {}", num_accounts);
    info!("  Order Data File: {}", order_data_file);

    // Initialize trade bot
    let bot = TradeBot::new(&node_url, num_accounts).await?;

    // Load orders from file
    let orders = load_orders(&order_data_file)?;

    // Fund accounts if not skipped
    if env::var("SKIP_FUNDING").unwrap_or_else(|_| "0".to_string()) != "1" {
        bot.fund_accounts().await?;
    } else {
        info!("⏭️  Skipping account funding (SKIP_FUNDING=1)");
    }

    // Wrap bot in Arc for shared ownership across workers
    let bot = Arc::new(bot);

    // Replay all orders with worker pool
    bot.replay_orders(orders).await?;

    info!("🎉 Trade bot completed successfully!");

    Ok(())
}
