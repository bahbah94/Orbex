use crate as pallet_orderbook;
use frame_support::derive_impl;
use frame_system::pallet;
use sp_runtime::BuildStorage;
use sp_runtime::traits::parameter_types;

type Block = frame_system::mocking::MockBlock<Test>;

#[frame_support::runtime]
mod runtime {
    use frame_support::runtime;

    #[runtime::runtime]
    #[runtime::derive(
        RuntimeCall,
        RuntimeError,
        RuntimeEvent,
        RuntimeOrigin,
        RuntimeFreezeReason,
        RuntimeHoldReason,
        RuntimeSlashReason,
        RuntimeLockId,
        RuntimeTask
    )]

    pub struct Test;

    #[runtime::pallet_index(0)]
    pub type System = frame_system::Pallet<Test>;

    #[runtime::pallet_index(1)]
    pub type Assets = pallet_assets::Pallet<Test>;

    #[runtime::pallet_index(2)]
    pub type Orderbook = pallet_orderbook::Pallet<Test>;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
}

impl pallet_assets::Config for Test {
    type WeightInfo = ();
}

parameter_types! {
    pub const MaxPendingOrders: u32 = 100;           // Max 100 pending orders per block in tests
    pub const MaxCancellationOrders: u32 = 50;       // Max 50 cancellations per block in tests
    pub const MaxOrders: u32 = 1000;                 // Max 1000 orders per price level in tests
    pub const MaxUserOrders: u32 = 100;              // Max 100 orders per user in tests
}

impl pallet_orderbook::Config for Test {
    type MaxPendingOrders = MaxPendingOrders;
    type MaxCancellationOrders = MaxCancellationOrders;
    type MaxOrders = MaxOrders;
    type MaxUserOrders = MaxUserOrders;
    type WeightInfo = pallet_orderbook::weights::SubstrateWeight<Test>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into()
}
