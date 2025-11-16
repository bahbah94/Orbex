#![cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::v2::*;

#[benchmarks]
mod benchmarks {
    use super::*;
    use crate::Pallet as Orderbook;
    use crate::types::{OrderSide, OrderType};
    use crate::{Call, Config, Pallet};
    use frame_support::assert_ok;
    use frame_support::traits::Hooks;
    use frame_system::RawOrigin;
    use pallet_assets::{ETH, USDT};

    // Type alias for cleaner code
    type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

    /// Helper to create a funded account with USDT and ETH
    fn funded_account<T: Config>(name: &'static str, index: u32) -> AccountIdOf<T> {
        let caller: AccountIdOf<T> = account(name, index, 0);

        // Deposit USDT (for buying)
        assert_ok!(pallet_assets::Pallet::<T>::deposit(
            RawOrigin::Signed(caller.clone()).into(),
            USDT,
            1_000_000_000u128,
        ));

        // Deposit ETH (for selling)
        assert_ok!(pallet_assets::Pallet::<T>::deposit(
            RawOrigin::Signed(caller.clone()).into(),
            ETH,
            1_000_000u128,
        ));

        caller
    }

    /// Helper to setup matching orders
    fn setup_matching_orders<T: Config>(num_bids: u32, num_asks: u32, price: u128) {
        for i in 0..num_bids {
            let buyer = funded_account::<T>("buyer", i);
            assert_ok!(Orderbook::<T>::place_order(
                RawOrigin::Signed(buyer).into(),
                OrderSide::Buy,
                price,
                10u128,
                OrderType::Limit
            ));
        }

        for i in 0..num_asks {
            let seller = funded_account::<T>("seller", i);
            assert_ok!(Orderbook::<T>::place_order(
                RawOrigin::Signed(seller).into(),
                OrderSide::Sell,
                price,
                10u128,
                OrderType::Limit
            ));
        }
    }

    /// Helper to setup non-matching orders
    fn setup_non_matching_orders<T: Config>(num_bids: u32, num_asks: u32) {
        for i in 0..num_bids {
            let buyer = funded_account::<T>("buyer", i);
            assert_ok!(Orderbook::<T>::place_order(
                RawOrigin::Signed(buyer).into(),
                OrderSide::Buy,
                90u128,
                10u128,
                OrderType::Limit
            ));
        }

        for i in 0..num_asks {
            let seller = funded_account::<T>("seller", i);
            assert_ok!(Orderbook::<T>::place_order(
                RawOrigin::Signed(seller).into(),
                OrderSide::Sell,
                110u128,
                10u128,
                OrderType::Limit
            ));
        }
    }

    /// Helper to setup cancellations
    fn setup_cancellations<T: Config>(num_cancellations: u32) {
        for i in 0..num_cancellations {
            let user = funded_account::<T>("user_cancel", i);

            // Get the CURRENT order_id before placing
            let order_id_before = Pallet::<T>::next_order_id();

            // Place order
            assert_ok!(Pallet::<T>::place_order(
                RawOrigin::Signed(user.clone()).into(),
                OrderSide::Buy,
                100u128,
                10u128,
                OrderType::Limit
            ));

            // The order_id that was just created is order_id_before
            // (because NextOrderId was incremented AFTER the order was placed)
            assert_ok!(Pallet::<T>::cancel_order(
                RawOrigin::Signed(user).into(),
                order_id_before, // Use the captured order_id
            ));
        }
    }

    // ========================================
    // EXTRINSIC BENCHMARKS
    // ========================================

    #[benchmark]
    fn place_order() {
        let caller = funded_account::<T>("caller", 0);

        #[extrinsic_call]
        place_order(
            RawOrigin::Signed(caller.clone()),
            OrderSide::Buy,
            100u128,
            10u128,
            OrderType::Limit,
        );

        assert_eq!(Orderbook::<T>::next_order_id(), 1);
    }

    #[benchmark]
    fn cancel_order() {
        let caller = funded_account::<T>("caller", 0);

        assert_ok!(Orderbook::<T>::place_order(
            RawOrigin::Signed(caller.clone()).into(),
            OrderSide::Buy,
            100u128,
            10u128,
            OrderType::Limit
        ));

        let order_id = 0;

        #[extrinsic_call]
        cancel_order(RawOrigin::Signed(caller.clone()), order_id);

        assert_eq!(Orderbook::<T>::get_pending_cancellations().len(), 1);
    }

    // ========================================
    // ON_FINALIZE BENCHMARKS
    // ========================================

    #[benchmark]
    fn on_finalize_empty() {
        #[block]
        {
            Orderbook::<T>::on_finalize(1u32.into());
        }
    }

    #[benchmark]
    fn on_finalize_with_matches(b: Linear<1, 50>, a: Linear<1, 50>) {
        setup_matching_orders::<T>(b, a, 100u128);

        #[block]
        {
            Orderbook::<T>::on_finalize(1u32.into());
        }

        assert!(Orderbook::<T>::next_trade_id() > 0);
    }

    #[benchmark]
    fn on_finalize_no_matches(b: Linear<1, 50>, a: Linear<1, 50>) {
        setup_non_matching_orders::<T>(b, a);

        #[block]
        {
            Orderbook::<T>::on_finalize(1u32.into());
        }

        assert!(Orderbook::<T>::get_bids_at_price(90).len() > 0);
        assert!(Orderbook::<T>::get_asks_at_price(110).len() > 0);
    }

    #[benchmark]
    fn on_finalize_with_cancellations(c: Linear<1, 50>) {
        setup_cancellations::<T>(c);

        #[block]
        {
            Orderbook::<T>::on_finalize(1u32.into());
        }

        assert_eq!(Orderbook::<T>::get_pending_cancellations().len(), 0);
    }

    #[benchmark]
    fn on_finalize_persistent_matching(p: Linear<1, 20>, n: Linear<1, 20>) {
        for i in 0..p {
            let seller = funded_account::<T>("persistent_seller", i);
            assert_ok!(Orderbook::<T>::place_order(
                RawOrigin::Signed(seller).into(),
                OrderSide::Sell,
                100u128,
                10u128,
                OrderType::Limit
            ));
        }

        Orderbook::<T>::on_finalize(1u32.into());

        for i in 0..n {
            let buyer = funded_account::<T>("new_buyer", i);
            assert_ok!(Orderbook::<T>::place_order(
                RawOrigin::Signed(buyer).into(),
                OrderSide::Buy,
                100u128,
                10u128,
                OrderType::Limit
            ));
        }

        #[block]
        {
            Orderbook::<T>::on_finalize(2u32.into());
        }

        assert!(Orderbook::<T>::next_trade_id() > 0);
    }

    #[benchmark]
    fn on_finalize_complex(m: Linear<1, 20>, n: Linear<1, 20>, c: Linear<1, 10>) {
        setup_matching_orders::<T>(m, m, 100u128);

        for i in 0..n {
            let buyer = funded_account::<T>("buyer_high", i);
            assert_ok!(Orderbook::<T>::place_order(
                RawOrigin::Signed(buyer).into(),
                OrderSide::Buy,
                90u128,
                10u128,
                OrderType::Limit
            ));
        }

        setup_cancellations::<T>(c);

        #[block]
        {
            Orderbook::<T>::on_finalize(1u32.into());
        }
    }

    impl_benchmark_test_suite!(Orderbook, crate::mock::new_test_ext(), crate::mock::Test);
}
