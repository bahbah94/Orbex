use crate::mock::*;
use crate::types::*;
use frame_support::{assert_ok, assert_noop, traits::Hooks};
use pallet_assets::{USDT, ETH};

// Simple u64 accounts for testing
fn alice() -> u64 {
    1
}

fn bob() -> u64 {
    2
}

fn charlie() -> u64 {
    3
}

// Helper to fund accounts
fn fund_account(account: u64, usdt: u128, eth: u128) {
    if usdt > 0 {
        assert_ok!(Assets::deposit(
            RuntimeOrigin::signed(account),
            USDT,
            usdt
        ));
    }
    if eth > 0 {
        assert_ok!(Assets::deposit(
            RuntimeOrigin::signed(account),
            ETH,
            eth
        ));
    }
}

// ============================================
// BASIC ORDER PLACEMENT TESTS
// ============================================

#[test]
fn test_place_buy_order_works() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        fund_account(alice, 10_000, 0);
        
        // Place buy order: 10 ETH @ $100
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(alice),
            OrderSide::Buy,
            100,
            10,
            OrderType::Limit,
        ));
        
        // Check order was created
        let order = Orderbook::get_order(0).expect("Order should exist");
        assert_eq!(order.trader, alice);
        assert_eq!(order.side, OrderSide::Buy);
        assert_eq!(order.price, 100);
        assert_eq!(order.quantity, 10);
        assert_eq!(order.status, OrderStatus::Open);
        
        // Check order ID incremented
        assert_eq!(Orderbook::next_order_id(), 1);
        
        // Check funds were locked (10 ETH * $100 = 1000 USDT)
        assert_eq!(Assets::get_free_balance(&alice, USDT), 9_000);
        assert_eq!(Assets::get_locked_balance(&alice, USDT), 1_000);
        
        // Check order was added to pending bids
        let pending = Orderbook::get_pending_bids_at_price(100);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0], 0);
    });
}

#[test]
fn test_place_sell_order_works() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        fund_account(alice, 0, 100);
        
        // Place sell order: 10 ETH @ $100
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(alice),
            OrderSide::Sell,
            100,
            10,
            OrderType::Limit,
        ));
        
        // Check order was created
        let order = Orderbook::get_order(0).expect("Order should exist");
        assert_eq!(order.side, OrderSide::Sell);
        assert_eq!(order.price, 100);
        
        // Check funds were locked
        assert_eq!(Assets::get_free_balance(&alice, ETH), 90);
        assert_eq!(Assets::get_locked_balance(&alice, ETH), 10);
        
        // Check order was added to pending asks
        let pending = Orderbook::get_pending_asks_at_price(100);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0], 0);
    });
}

#[test]
fn test_place_multiple_orders_same_price() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        let bob = bob();
        fund_account(alice, 10_000, 0);
        fund_account(bob, 10_000, 0);
        
        // Alice places buy order
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(alice),
            OrderSide::Buy,
            100,
            5,
            OrderType::Limit,
        ));
        
        // Bob places buy order at same price
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(bob),
            OrderSide::Buy,
            100,
            10,
            OrderType::Limit,
        ));
        
        // Check both orders in pending bids
        let pending = Orderbook::get_pending_bids_at_price(100);
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0], 0); // Alice first (FIFO)
        assert_eq!(pending[1], 1); // Bob second
        
        // Check next order ID
        assert_eq!(Orderbook::next_order_id(), 2);
    });
}

#[test]
fn test_place_market_order() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        fund_account(alice, 10_000, 0);
        
        // Place market buy order (price is ignored)
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(alice),
            OrderSide::Buy,
            200, 
            10,
            OrderType::Market,
        ));
        
        let order = Orderbook::get_order(0).expect("Order should exist");
        assert_eq!(order.order_type, OrderType::Market);
    });
}

// ============================================
// VALIDATION TESTS
// ============================================

#[test]
fn test_place_order_invalid_price_fails() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        fund_account(alice, 10_000, 0);
        
        assert_noop!(
            Orderbook::place_order(
                RuntimeOrigin::signed(alice),
                OrderSide::Buy,
                0, // Invalid price for limit order
                10,
                OrderType::Limit,
            ),
            crate::Error::<Test>::InvalidPrice
        );
    });
}

#[test]
fn test_place_order_invalid_quantity_fails() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        fund_account(alice, 10_000, 0);
        
        assert_noop!(
            Orderbook::place_order(
                RuntimeOrigin::signed(alice),
                OrderSide::Buy,
                100,
                0, // Invalid quantity
                OrderType::Limit,
            ),
            crate::Error::<Test>::InvalidQuantity
        );
    });
}

#[test]
fn test_place_order_insufficient_balance_fails() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        fund_account(alice, 500, 0); // Only 500 USDT
        
        // Try to buy 10 ETH @ $100 (needs 1000 USDT)
        assert_noop!(
            Orderbook::place_order(
                RuntimeOrigin::signed(alice),
                OrderSide::Buy,
                100,
                10,
                OrderType::Limit,
            ),
            pallet_assets::Error::<Test>::InsufficientFreeBalance
        );
    });
}

#[test]
fn test_place_order_arithmetic_overflow() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        fund_account(alice, u128::MAX, 0);
        
        // Try to create order that would overflow
        assert_noop!(
            Orderbook::place_order(
                RuntimeOrigin::signed(alice),
                OrderSide::Buy,
                u128::MAX,
                u128::MAX, // This would overflow when multiplied
                OrderType::Limit,
            ),
            crate::Error::<Test>::ArithmeticOverflow
        );
    });
}

// ============================================
// CANCELLATION TESTS
// ============================================

#[test]
fn test_cancel_order_works() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        fund_account(alice, 10_000, 0);
        
        // Place order
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(alice),
            OrderSide::Buy,
            100,
            10,
            OrderType::Limit,
        ));
        
        // Cancel order
        assert_ok!(Orderbook::cancel_order(
            RuntimeOrigin::signed(alice),
            0, // order_id
        ));
        
        // Check cancellation was queued
        let cancellations = Orderbook::get_pending_cancellations();
        assert_eq!(cancellations.len(), 1);
        assert_eq!(cancellations[0], 0);
    });
}

#[test]
fn test_cancel_order_not_owner_fails() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        let bob = bob();
        fund_account(alice, 10_000, 0);
        
        // Alice places order
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(alice),
            OrderSide::Buy,
            100,
            10,
            OrderType::Limit,
        ));
        
        // Bob tries to cancel Alice's order - should fail
        assert_noop!(
            Orderbook::cancel_order(
                RuntimeOrigin::signed(bob),
                0,
            ),
            crate::Error::<Test>::NotOrderOwner
        );
    });
}

#[test]
fn test_cancel_nonexistent_order_fails() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        
        assert_noop!(
            Orderbook::cancel_order(
                RuntimeOrigin::signed(alice),
                999, // Doesn't exist
            ),
            crate::Error::<Test>::OrderNotFound
        );
    });
}

#[test]
fn test_cancel_multiple_orders() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        fund_account(alice, 10_000, 0);
        
        // Place 3 orders
        for i in 0..3 {
            assert_ok!(Orderbook::place_order(
                RuntimeOrigin::signed(alice),
                OrderSide::Buy,
                100 + i as u128,
                10,
                OrderType::Limit,
            ));
        }
        
        // Cancel first and third order
        assert_ok!(Orderbook::cancel_order(RuntimeOrigin::signed(alice), 0));
        assert_ok!(Orderbook::cancel_order(RuntimeOrigin::signed(alice), 2));
        
        // Check both cancellations queued
        let cancellations = Orderbook::get_pending_cancellations();
        assert_eq!(cancellations.len(), 2);
        assert_eq!(cancellations[0], 0);
        assert_eq!(cancellations[1], 2);
    });
}

// ============================================
// EDGE CASE TESTS
// ============================================

#[test]
fn test_place_order_with_exact_balance() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        fund_account(alice, 1_000, 0); // Exactly 1000 USDT
        
        // Buy exactly what we can afford
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(alice),
            OrderSide::Buy,
            100,
            10, // Exactly 1000 USDT needed
            OrderType::Limit,
        ));
        
        // Should have locked all funds
        assert_eq!(Assets::get_free_balance(&alice, USDT), 0);
        assert_eq!(Assets::get_locked_balance(&alice, USDT), 1_000);
    });
}

#[test]
fn test_place_order_one_wei_short_fails() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        fund_account(alice, 999, 0); // One less than needed
        
        assert_noop!(
            Orderbook::place_order(
                RuntimeOrigin::signed(alice),
                OrderSide::Buy,
                100,
                10, // Needs 1000 USDT
                OrderType::Limit,
            ),
            pallet_assets::Error::<Test>::InsufficientFreeBalance
        );
    });
}

#[test]
fn test_sequential_orders_increment_ids() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        fund_account(alice, 100_000, 1000);
        
        // Place 5 orders
        for i in 0..5 {
            assert_ok!(Orderbook::place_order(
                RuntimeOrigin::signed(alice),
                if i % 2 == 0 { OrderSide::Buy } else { OrderSide::Sell },
                100,
                1,
                OrderType::Limit,
            ));
            
            // Check ID incremented correctly
            assert_eq!(Orderbook::next_order_id(), i + 1);
            
            // Check order exists with correct ID
            let order = Orderbook::get_order(i).expect("Order should exist");
            assert_eq!(order.order_id, i);
        }
    });
}

#[test]
fn test_large_order_values() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        fund_account(alice, 1_000_000_000, 0); // 1 billion USDT
        
        // Place large order
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(alice),
            OrderSide::Buy,
            10_000, // $10,000 per ETH
            100_000, // 100k ETH
            OrderType::Limit,
        ));
        
        // Check huge amount locked (10k * 100k = 1 billion)
        assert_eq!(Assets::get_locked_balance(&alice, USDT), 1_000_000_000);
    });
}

#[test]
fn test_different_users_different_orders() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        let bob = bob();
        let charlie = charlie();
        
        fund_account(alice, 10_000, 0);
        fund_account(bob, 0, 100);
        fund_account(charlie, 5_000, 50);
        
        // Each user places different order
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(alice),
            OrderSide::Buy,
            100,
            10,
            OrderType::Limit,
        ));
        
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(bob),
            OrderSide::Sell,
            105,
            20,
            OrderType::Limit,
        ));
        
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(charlie),
            OrderSide::Buy,
            98,
            5,
            OrderType::Limit,
        ));
        
        // Verify each order has correct owner
        assert_eq!(Orderbook::get_order(0).unwrap().trader, alice);
        assert_eq!(Orderbook::get_order(1).unwrap().trader, bob);
        assert_eq!(Orderbook::get_order(2).unwrap().trader, charlie);
        
        // Verify 3 orders created
        assert_eq!(Orderbook::next_order_id(), 3);
    });
}


#[test]
fn test_simple_buy_sell_match() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        let bob = bob();
        
        // Setup: Give Alice USDT, Bob ETH
        fund_account(alice, 10_000, 0);
        fund_account(bob, 0, 100);
        
        // Alice: Buy 10 ETH @ $100
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(alice),
            OrderSide::Buy,
            100,
            10,
            OrderType::Limit,
        ));
        
        // Bob: Sell 10 ETH @ $100
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(bob),
            OrderSide::Sell,
            100,
            10,
            OrderType::Limit,
        ));
        
        // Both orders pending
        assert_eq!(Orderbook::get_pending_bids_at_price(100).len(), 1);
        assert_eq!(Orderbook::get_pending_asks_at_price(100).len(), 1);
        
        // Trigger matching by advancing to next block
        System::set_block_number(1);
        Orderbook::on_finalize(1);
        
        // Verify trade executed
        let trade = Orderbook::get_trade(0).expect("Trade should exist");
        assert_eq!(trade.buyer, alice);
        assert_eq!(trade.seller, bob);
        assert_eq!(trade.price, 100);
        assert_eq!(trade.quantity, 10);
        
        // Verify balances after settlement
        // Alice: spent 1000 USDT, got 10 ETH
        assert_eq!(Assets::get_free_balance(&alice, USDT), 9_000);
        assert_eq!(Assets::get_free_balance(&alice, ETH), 10);
        assert_eq!(Assets::get_locked_balance(&alice, USDT), 0);
        assert_eq!(Assets::get_locked_balance(&alice, ETH), 0);
        
        // Bob: got 1000 USDT, spent 10 ETH
        assert_eq!(Assets::get_free_balance(&bob, USDT), 1_000);
        assert_eq!(Assets::get_free_balance(&bob, ETH), 90);
        assert_eq!(Assets::get_locked_balance(&bob, USDT), 0);
        assert_eq!(Assets::get_locked_balance(&bob, ETH), 0);
        
        // Verify orders are filled
        let alice_order = Orderbook::get_order(0).unwrap();
        assert_eq!(alice_order.status, OrderStatus::Filled);
        assert_eq!(alice_order.filled_quantity, 10);
        
        let bob_order = Orderbook::get_order(1).unwrap();
        assert_eq!(bob_order.status, OrderStatus::Filled);
        assert_eq!(bob_order.filled_quantity, 10);
        
        // Verify trade ID incremented
        assert_eq!(Orderbook::next_trade_id(), 1);
    });
}

#[test]
#[test]
fn test_partial_fill_matching_debug() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        let bob = bob();
        
        fund_account(alice, 10_000, 0);
        fund_account(bob, 0, 100);
        
        println!("=== Initial state ===");
        println!("Alice USDT: {}", Assets::get_free_balance(&alice, USDT));
        println!("Bob ETH: {}", Assets::get_free_balance(&bob, ETH));
        
        // Alice: Buy 10 ETH @ $100
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(alice),
            OrderSide::Buy,
            100,
            10,
            OrderType::Limit,
        ));
        
        println!("\n=== After Alice order ===");
        println!("Alice free USDT: {}", Assets::get_free_balance(&alice, USDT));
        println!("Alice locked USDT: {}", Assets::get_locked_balance(&alice, USDT));
        
        // Bob: Sell only 5 ETH @ $100
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(bob),
            OrderSide::Sell,
            100,
            5,
            OrderType::Limit,
        ));
        
        println!("\n=== After Bob order ===");
        println!("Bob free ETH: {}", Assets::get_free_balance(&bob, ETH));
        println!("Bob locked ETH: {}", Assets::get_locked_balance(&bob, ETH));
        
        <Orderbook as Hooks<u64>>::on_finalize(1);
        
        println!("\n=== After matching ===");
        let alice_order = Orderbook::get_order(0).unwrap();
        println!("Alice order status: {:?}", alice_order.status);
        println!("Alice filled: {}/{}", alice_order.filled_quantity, alice_order.quantity);
        
        let bob_order = Orderbook::get_order(1).unwrap();
        println!("Bob order status: {:?}", bob_order.status);
        println!("Bob filled: {}/{}", bob_order.filled_quantity, bob_order.quantity);
        
        println!("\n=== Final balances ===");
        println!("Alice free USDT: {}", Assets::get_free_balance(&alice, USDT));
        println!("Alice locked USDT: {}", Assets::get_locked_balance(&alice, USDT));
        println!("Alice free ETH: {}", Assets::get_free_balance(&alice, ETH));
        println!("Alice locked ETH: {}", Assets::get_locked_balance(&alice, ETH));
        
        println!("Bob free USDT: {}", Assets::get_free_balance(&bob, USDT));
        println!("Bob locked USDT: {}", Assets::get_locked_balance(&bob, USDT));
        println!("Bob free ETH: {}", Assets::get_free_balance(&bob, ETH));
        println!("Bob locked ETH: {}", Assets::get_locked_balance(&bob, ETH));
        
        let trade = Orderbook::get_trade(0).unwrap();
        println!("\nTrade: {} ETH @ ${}", trade.quantity, trade.price);
    });
}
#[test]
fn test_price_time_priority_fifo() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        let bob = bob();
        let charlie = charlie();
        
        fund_account(alice, 0, 100);
        fund_account(bob, 0, 100);
        fund_account(charlie, 10_000, 0);
        
        // Alice: Sell 5 ETH @ $100 (FIRST)
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(alice),
            OrderSide::Sell,
            100,
            5,
            OrderType::Limit,
        ));
        
        // Bob: Sell 5 ETH @ $100 (SECOND - same price, later time)
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(bob),
            OrderSide::Sell,
            100,
            5,
            OrderType::Limit,
        ));
        
        // Charlie: Buy 5 ETH @ $100
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(charlie),
            OrderSide::Buy,
            100,
            5,
            OrderType::Limit,
        ));
        
        // Trigger matching
        System::set_block_number(1);
        Orderbook::on_finalize(1);
        
        // Should match with Alice (FIFO - first in, first out)
        let trade = Orderbook::get_trade(0).unwrap();
        assert_eq!(trade.seller, alice); // Alice matched, not Bob
        assert_eq!(trade.buyer, charlie);
        
        // Alice's order filled, Bob's still open
        let alice_order = Orderbook::get_order(0).unwrap();
        assert_eq!(alice_order.status, OrderStatus::Filled);
        
        let bob_order = Orderbook::get_order(1).unwrap();
        assert_eq!(bob_order.status, OrderStatus::Open); // Still waiting!
        
        // Verify Alice got paid
        assert_eq!(Assets::get_free_balance(&alice, USDT), 500);
        assert_eq!(Assets::get_free_balance(&alice, ETH), 95);
        
        // Bob didn't trade yet
        assert_eq!(Assets::get_free_balance(&bob, USDT), 0);
        assert_eq!(Assets::get_locked_balance(&bob, ETH), 5); // Still locked
    });
}

#[test]
fn test_no_match_price_spread() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        let bob = bob();
        
        fund_account(alice, 10_000, 0);
        fund_account(bob, 0, 100);
        
        // Alice: Buy @ $95
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(alice),
            OrderSide::Buy,
            95,
            10,
            OrderType::Limit,
        ));
        
        // Bob: Sell @ $105 (no match - spread too wide)
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(bob),
            OrderSide::Sell,
            105,
            10,
            OrderType::Limit,
        ));
        
        // Trigger matching
        System::set_block_number(1);
        Orderbook::on_finalize(1);
        
        // No trades should execute
        assert!(Orderbook::get_trade(0).is_none());
        
        // Both orders should remain open
        let alice_order = Orderbook::get_order(0).unwrap();
        assert_eq!(alice_order.status, OrderStatus::Open);
        assert_eq!(alice_order.filled_quantity, 0);
        
        let bob_order = Orderbook::get_order(1).unwrap();
        assert_eq!(bob_order.status, OrderStatus::Open);
        assert_eq!(bob_order.filled_quantity, 0);
        
        // Funds still locked
        assert_eq!(Assets::get_locked_balance(&alice, USDT), 950);
        assert_eq!(Assets::get_locked_balance(&bob, ETH), 10);
    });
}

#[test]
fn test_multiple_trades_same_block() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        let bob = bob();
        let charlie = charlie();
        
        fund_account(alice, 10_000, 0);
        fund_account(bob, 0, 50);
        fund_account(charlie, 0, 50);
        
        // Alice: Buy 20 ETH @ $100
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(alice),
            OrderSide::Buy,
            100,
            20,
            OrderType::Limit,
        ));
        
        // Bob: Sell 10 ETH @ $100
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(bob),
            OrderSide::Sell,
            100,
            10,
            OrderType::Limit,
        ));
        
        // Charlie: Sell 10 ETH @ $100
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(charlie),
            OrderSide::Sell,
            100,
            10,
            OrderType::Limit,
        ));
        
        // Trigger matching
        System::set_block_number(1);
        Orderbook::on_finalize(1);
        
        // Should create 2 trades (Alice with Bob, Alice with Charlie)
        assert!(Orderbook::get_trade(0).is_some());
        assert!(Orderbook::get_trade(1).is_some());
        
        let trade1 = Orderbook::get_trade(0).unwrap();
        let trade2 = Orderbook::get_trade(1).unwrap();
        
        // Both trades with Alice as buyer
        assert_eq!(trade1.buyer, alice);
        assert_eq!(trade2.buyer, alice);
        
        // Bob and Charlie as sellers
        assert!(trade1.seller == bob || trade2.seller == bob);
        assert!(trade1.seller == charlie || trade2.seller == charlie);
        
        // Alice's order should be fully filled (20 ETH total)
        let alice_order = Orderbook::get_order(0).unwrap();
        assert_eq!(alice_order.status, OrderStatus::Filled);
        assert_eq!(alice_order.filled_quantity, 20);
        
        // Alice should have 20 ETH, spent 2000 USDT
        assert_eq!(Assets::get_free_balance(&alice, ETH), 20);
        assert_eq!(Assets::get_free_balance(&alice, USDT), 8_000);
        assert_eq!(Assets::get_locked_balance(&alice, USDT), 0);
        
        // Bob got 1000 USDT
        assert_eq!(Assets::get_free_balance(&bob, USDT), 1_000);
        
        // Charlie got 1000 USDT
        assert_eq!(Assets::get_free_balance(&charlie, USDT), 1_000);
    });
}

#[test]
fn test_cancellation_unlocks_funds() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        fund_account(alice, 10_000, 0);
        
        // Place order
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(alice),
            OrderSide::Buy,
            100,
            10,
            OrderType::Limit,
        ));
        
        // Verify funds locked
        assert_eq!(Assets::get_locked_balance(&alice, USDT), 1_000);
        
        // Cancel order
        assert_ok!(Orderbook::cancel_order(
            RuntimeOrigin::signed(alice),
            0,
        ));
        
        // Trigger finalization to process cancellation
        System::set_block_number(1);
        Orderbook::on_finalize(1);
        
        // Verify funds unlocked
        assert_eq!(Assets::get_free_balance(&alice, USDT), 10_000);
        assert_eq!(Assets::get_locked_balance(&alice, USDT), 0);
        
        // Verify order cancelled
        let order = Orderbook::get_order(0).unwrap();
        assert_eq!(order.status, OrderStatus::Cancelled);
    });
}

#[test]
#[test]
fn test_market_order_matches_best_price() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        let bob = bob();
        
        fund_account(alice, 0, 100);
        fund_account(bob, 10_000, 0);
        
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(alice),
            OrderSide::Sell,
            95,
            10,
            OrderType::Limit,
        ));
        
        // For batch matching, market orders still use the price for locking funds
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(bob),
            OrderSide::Buy,
            95,  // Match at same price
            10,
            OrderType::Market,
        ));
        
        <Orderbook as Hooks<u64>>::on_finalize(1);
        
        let trade = Orderbook::get_trade(0).unwrap();
        assert_eq!(trade.price, 95);
        
        assert_eq!(Assets::get_free_balance(&bob, USDT), 9_050);
        assert_eq!(Assets::get_free_balance(&bob, ETH), 10);
    });
}
#[test]
fn test_simple_buy_sell_match_debug() {
    new_test_ext().execute_with(|| {
        let alice = alice();
        let bob = bob();
        
        fund_account(alice, 10_000, 0);
        fund_account(bob, 0, 100);
        
        println!("=== Before orders ===");
        println!("Alice USDT: {}", Assets::get_free_balance(&alice, USDT));
        println!("Bob ETH: {}", Assets::get_free_balance(&bob, ETH));
        
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(alice),
            OrderSide::Buy,
            100,
            10,
            OrderType::Limit,
        ));
        
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(bob),
            OrderSide::Sell,
            100,
            10,
            OrderType::Limit,
        ));
        
        println!("=== After orders placed ===");
        println!("Pending bids at 100: {:?}", Orderbook::get_pending_bids_at_price(100));
        println!("Pending asks at 100: {:?}", Orderbook::get_pending_asks_at_price(100));
        println!("Order 0: {:?}", Orderbook::get_order(0));
        println!("Order 1: {:?}", Orderbook::get_order(1));
        
        // Call on_finalize
        println!("=== Calling on_finalize ===");
        <Orderbook as Hooks<u64>>::on_finalize(1);
        
        println!("=== After on_finalize ===");
        println!("Order 0: {:?}", Orderbook::get_order(0));
        println!("Order 1: {:?}", Orderbook::get_order(1));
        println!("Trade 0: {:?}", Orderbook::get_trade(0));
        println!("Alice USDT: {}", Assets::get_free_balance(&alice, USDT));
        println!("Alice ETH: {}", Assets::get_free_balance(&alice, ETH));
        println!("Bob USDT: {}", Assets::get_free_balance(&bob, USDT));
        println!("Bob ETH: {}", Assets::get_free_balance(&bob, ETH));
    });
}