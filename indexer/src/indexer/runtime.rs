// Generate types from metadata at compile time
#[subxt::subxt(runtime_metadata_path = "../metadata.scale")]
pub mod polkadot {}

pub use polkadot::orderbook::events::TradeExecuted;
