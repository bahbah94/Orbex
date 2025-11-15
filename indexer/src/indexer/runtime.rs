// Generate types from metadata at compile time
#[subxt::subxt(runtime_metadata_path = "../metadata.scale")]
pub mod polkadot {}

pub use polkadot::orderbook::events::OrderCancelled;
pub use polkadot::orderbook::events::OrderFilled;
pub use polkadot::orderbook::events::OrderPartiallyFilled;
pub use polkadot::orderbook::events::OrderPlaced;
pub use polkadot::orderbook::events::TradeExecuted;
impl std::fmt::Display for polkadot::runtime_types::pallet_orderbook::types::OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let side_str = match self {
            polkadot::runtime_types::pallet_orderbook::types::OrderSide::Buy => "Buy",
            polkadot::runtime_types::pallet_orderbook::types::OrderSide::Sell => "Sell",
        };
        write!(f, "{}", side_str)
    }
}
