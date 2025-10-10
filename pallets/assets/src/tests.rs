// pallets/assets/src/tests.rs

use crate::{mock::*, Error, Event, USDT, ETH};
use frame_support::{assert_noop, assert_ok};

#[test]
fn deposit_works() {
    new_test_ext().execute_with(|| {
        // Go to block 1 (for events)
        System::set_block_number(1);
        
        // Deposit 1000 USDT
        assert_ok!(Assets::deposit(RuntimeOrigin::signed(1), USDT, 1000));
        
        // Check balance
        assert_eq!(Assets::get_free_balance(&1, USDT), 1000);
        
        // Check event
        System::assert_has_event(
            Event::Deposited { 
                user: 1, 
                asset_id: USDT, 
                amount: 1000 
            }.into()
        );
    });
}

#[test]
fn deposit_zero_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Assets::deposit(RuntimeOrigin::signed(1), USDT, 0),
            Error::<Test>::AmountZero
        );
    });
}

#[test]
fn deposit_invalid_asset_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Assets::deposit(RuntimeOrigin::signed(1), 999, 100),
            Error::<Test>::InvalidAsset
        );
    });
}

#[test]
fn withdraw_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // Setup: deposit first
        assert_ok!(Assets::deposit(RuntimeOrigin::signed(1), USDT, 1000));
        
        // Withdraw 300
        assert_ok!(Assets::withdraw(RuntimeOrigin::signed(1), USDT, 300));
        
        // Check balance
        assert_eq!(Assets::get_free_balance(&1, USDT), 700);
    });
}

#[test]
fn withdraw_insufficient_balance_fails() {
    new_test_ext().execute_with(|| {
        // No deposit, try to withdraw
        assert_noop!(
            Assets::withdraw(RuntimeOrigin::signed(1), USDT, 100),
            Error::<Test>::InsufficientFreeBalance
        );
    });
}

#[test]
fn lock_and_unlock_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // Deposit 1000
        assert_ok!(Assets::deposit(RuntimeOrigin::signed(1), ETH, 1000));
        
        // Lock 400
        assert_ok!(Assets::lock_funds(&1, ETH, 400));
        assert_eq!(Assets::get_free_balance(&1, ETH), 600);
        assert_eq!(Assets::get_locked_balance(&1, ETH), 400);
        
        // Unlock 200
        assert_ok!(Assets::unlock_funds(&1, ETH, 200));
        assert_eq!(Assets::get_free_balance(&1, ETH), 800);
        assert_eq!(Assets::get_locked_balance(&1, ETH), 200);
    });
}

#[test]
fn transfer_locked_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // User 1 deposits and locks
        assert_ok!(Assets::deposit(RuntimeOrigin::signed(1), USDT, 1000));
        assert_ok!(Assets::lock_funds(&1, USDT, 500));
        
        // Transfer locked from user 1 to user 2
        assert_ok!(Assets::transfer_locked(&1, &2, USDT, 300));
        
        // Check balances
        assert_eq!(Assets::get_locked_balance(&1, USDT), 200);  // 500 - 300
        assert_eq!(Assets::get_locked_balance(&2, USDT), 300);    // Received as free
    });
}