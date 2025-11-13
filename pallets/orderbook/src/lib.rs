#![cfg_attr(not(feature = "std"), no_std)]
#![allow(ambiguous_glob_reexports)]
mod engine;
pub mod types;
pub use pallet::*;
//pub use crate::types;
//pub use pallet_assets::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    //use std::intrinsics::saturating_add;

    use core::u32;

    //use super::*;
    use crate::{
        engine::*,
        types::{Amount, Order, OrderId, OrderSide, OrderStatus, OrderType, Trade, TradeId},
        weights::WeightInfo,
    };

    use assets::{ETH, USDT};
    use frame_support::{pallet_prelude::*, Blake2_128Concat};
    use frame_system::pallet_prelude::{OriginFor, *};
    use pallet_assets as assets;
    use sp_core::Get;
    use sp_std::{collections::btree_map::BTreeMap, vec::Vec};
    //use assets::*;
    //use sp_runtime::legacy::byte_sized_error::DispatchError;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_assets::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        // maximum orders at any price level not sure if needed. this will be on the blockOrders cache
        #[pallet::constant]
        type MaxPendingOrders: Get<u32>;

        //keeping this to prevent DDoS attacks for cancelling too many orders
        #[pallet::constant]
        type MaxCancellationOrders: Get<u32>;

        #[pallet::constant]
        type MaxOrders: Get<u32>;

        #[pallet::constant]
        type MaxUserOrders: Get<u32>;

        type WeightInfo: WeightInfo;
    }

    // ===========================
    // Persisten storage
    // ===========================

    // not sure if this needed yet, so just keeping it
    #[pallet::storage]
    pub type Orders<T: Config> = StorageMap<_, Blake2_128Concat, OrderId, Order<T>, OptionQuery>;

    #[pallet::storage]
    pub type Trades<T: Config> = StorageMap<_, Blake2_128Concat, TradeId, Trade<T>, OptionQuery>;

    #[pallet::storage]
    pub type Bids<T: Config> =
        StorageMap<_, Blake2_128Concat, Amount, BoundedVec<OrderId, T::MaxOrders>, ValueQuery>;

    #[pallet::storage]
    pub type Asks<T: Config> =
        StorageMap<_, Blake2_128Concat, Amount, BoundedVec<OrderId, T::MaxOrders>, ValueQuery>;

    // ===========================
    // Cache
    // ===========================

    #[pallet::storage]
    pub type PendingAsks<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        Amount,
        BoundedVec<OrderId, T::MaxPendingOrders>,
        ValueQuery,
    >;

    #[pallet::storage]
    pub type PendingBids<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        Amount,
        BoundedVec<OrderId, T::MaxPendingOrders>,
        ValueQuery,
    >;

    #[pallet::storage]
    pub type PendingCancellations<T: Config> =
        StorageValue<_, BoundedVec<OrderId, T::MaxCancellationOrders>, ValueQuery>;

    //Keeping this so that users can easily access their orders
    #[pallet::storage]
    pub type UserOrders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<OrderId, T::MaxUserOrders>,
        ValueQuery,
    >;

    #[pallet::storage]
    pub type NextOrderId<T: Config> = StorageValue<_, OrderId, ValueQuery>;

    #[pallet::storage]
    pub type NextTradeId<T: Config> = StorageValue<_, TradeId, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        OrderPlaced {
            order_id: OrderId,
            side: OrderSide,
            asset_id: u32,
            price: Amount,
            quantity: Amount,
        },
        TradeExecuted {
            trade_id: TradeId,
            buy_order_id: OrderId,
            sell_order_id: OrderId,
            buyer: T::AccountId,
            seller: T::AccountId,
            price: Amount,
            quantity: Amount,
        },
        OrderCancelled {
            order_id: OrderId,
            trader: T::AccountId,
        },
        OrderFilled {
            order_id: OrderId,
            trader: T::AccountId,
        },
        OrderPartiallyFilled {
            order_id: OrderId,
            trader: T::AccountId,
            filled_quantity: Amount,
            remaining_quantity: Amount,
        },
        // we are putting this event, so that we know its requested but it could not be processed perhaps
        CancellationRequested {
            order_id: OrderId,
            trader: T::AccountId,
        },
        MatchingCompleted {
            total_trades: u32,
            total_volume: Amount,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Order not found
        OrderNotFound,

        /// Not the order owner
        NotOrderOwner,

        /// Order not active
        OrderNotActive,

        /// Price must be > 0
        InvalidPrice,

        /// Quantity must be > 0
        InvalidQuantity,

        /// Insufficient balance
        InsufficientBalance,

        /// Too many pending orders this block
        TooManyPendingOrders,

        //too many user orders
        TooManyUserOrders,

        /// Too many pending cancellations this block
        TooManyPendingCancellations,

        /// Arithmetic overflow
        ArithmeticOverflow,

        /// Arithmetic underflow
        ArithmeticUnderflow,

        /// Failed to unreserve funds
        FailedToUnreserveFunds,

        /// No matching orders
        NoMatchingOrders,
    }

    // ========================================
    // HOOKS FOR MATCHING
    // ========================================

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// Calculate weight based on pending work
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            // Count pending orders quickly
            let mut total_pending = 0u32;

            // Quick count of pending bids/asks
            for (_, orders) in PendingBids::<T>::iter() {
                total_pending = total_pending.saturating_add(orders.len() as u32);
            }
            for (_, orders) in PendingAsks::<T>::iter() {
                total_pending = total_pending.saturating_add(orders.len() as u32);
            }

            let cancellations = PendingCancellations::<T>::get().len() as u32;

            // Return worst-case weight for safety
            if total_pending > 0 || cancellations > 0 {
                // Use complex scenario as upper bound
                <T as Config>::WeightInfo::on_finalize_complex(
                    total_pending.min(20),
                    total_pending.min(20),
                    cancellations.min(10),
                )
            } else {
                <T as Config>::WeightInfo::on_finalize_empty()
            }
        }

        // on finalize
        fn on_finalize(_n: BlockNumberFor<T>) {
            let mut orders_map = BTreeMap::new();

            //load all orders, will need to modify for sure
            for (order_id, order) in Orders::<T>::iter() {
                orders_map.insert(order_id, order);
            }

            //================================
            // These will load the temp caches
            //================================

            // Load pending bids
            let mut pending_bids = BTreeMap::new();
            for (price, order_ids) in PendingBids::<T>::iter() {
                pending_bids.insert(price, order_ids.into_inner());
            }

            // Load pending asks
            let mut pending_asks = BTreeMap::new();
            for (price, order_ids) in PendingAsks::<T>::iter() {
                pending_asks.insert(price, order_ids.into_inner());
            }

            // Load persistent bids
            let mut persistent_bids = BTreeMap::new();
            for (price, order_ids) in Bids::<T>::iter() {
                persistent_bids.insert(price, order_ids.into_inner());
            }

            // Load persistent asks
            let mut persistent_asks = BTreeMap::new();
            for (price, order_ids) in Asks::<T>::iter() {
                persistent_asks.insert(price, order_ids.into_inner());
            }

            let cancellations = PendingCancellations::<T>::get();
            if !cancellations.is_empty() {
                let _ = process_cancellations::<T>(
                    cancellations.into_inner(),
                    &mut persistent_bids,
                    &mut persistent_asks,
                    &mut orders_map,
                );
            }

            let mut all_trades: Vec<Trade<T>> = Vec::new();

            // here we are matching first only from the temp cache
            let (pending_trades, unmatched) =
                match match_pending_internal(pending_bids, pending_asks, &mut orders_map) {
                    Ok(result) => result,
                    Err(_) => (Vec::new(), Vec::new()),
                };

            all_trades.extend(pending_trades);

            if !unmatched.is_empty() {
                let persistent_trades = match match_persistent_storage(
                    &mut persistent_bids,
                    &mut persistent_asks,
                    unmatched,
                    &mut orders_map,
                ) {
                    Ok(trades) => trades,
                    Err(_) => Vec::new(),
                };

                all_trades.extend(persistent_trades);
            }

            // At this point, we have in memory done all necessary transactions
            // Now we need to adjust order/money management
            let mut total_volume = 0u128;

            for trade in all_trades.iter_mut() {
                // Set trade_id
                let trade_id = NextTradeId::<T>::get();
                trade.trade_id = trade_id;

                // Transfer USDT from buyer to seller
                let usdt_amount = trade.price.saturating_mul(trade.quantity);
                let _ = assets::Pallet::<T>::transfer_locked(
                    &trade.buyer,
                    &trade.seller,
                    USDT,
                    usdt_amount,
                );

                // Transfer ETH from seller to buyer
                let _ = assets::Pallet::<T>::transfer_locked(
                    &trade.seller,
                    &trade.buyer,
                    ETH,
                    trade.quantity,
                );

                // Unlock funds for both parties(NOt required i realized that transfer_locked alredy transfer
                //to free balance, unlocking might unlock some other things not in the trade)
                //let _ = assets::Pallet::<T>::unlock_funds(&trade.seller, USDT, usdt_amount);
                //let _ = assets::Pallet::<T>::unlock_funds(&trade.buyer, ETH, trade.quantity);

                // Store trade
                Trades::<T>::insert(trade_id, trade.clone());
                NextTradeId::<T>::put(trade_id + 1);

                // Emit event
                Self::deposit_event(Event::TradeExecuted {
                    trade_id,
                    buy_order_id: trade.buy_order_id,
                    sell_order_id: trade.sell_order_id,
                    buyer: trade.buyer.clone(),
                    seller: trade.seller.clone(),
                    price: trade.price,
                    quantity: trade.quantity,
                });

                total_volume = total_volume.saturating_add(usdt_amount);
            }

            // Now we need to unlock funds which are cancelled
            for (order_id, order) in orders_map.iter() {
                if order.status == OrderStatus::Cancelled {
                    let remaining = order.quantity.saturating_sub(order.filled_quantity);

                    if remaining > 0 {
                        let (asset, amount) = match order.side {
                            OrderSide::Buy => {
                                let total = order.price.saturating_mul(remaining);
                                (USDT, total)
                            }
                            OrderSide::Sell => (ETH, remaining),
                        };

                        let _ = assets::Pallet::<T>::unlock_funds(&order.trader, asset, amount);

                        Self::deposit_event(Event::OrderCancelled {
                            order_id: *order_id,
                            trader: order.trader.clone(),
                        });
                    }
                }
            }

            // Emit events for filled/partially filled:
            for (order_id, order) in orders_map.iter() {
                Orders::<T>::insert(order_id, order);

                // Emit events for filled/partially filled orders
                if order.status == OrderStatus::Filled {
                    Self::deposit_event(Event::OrderFilled {
                        order_id: *order_id,
                        trader: order.trader.clone(),
                    });
                    Orders::<T>::remove(order_id);
                } else if order.status == OrderStatus::PartiallyFilled {
                    let remaining = order.quantity.saturating_sub(order.filled_quantity);
                    Self::deposit_event(Event::OrderPartiallyFilled {
                        order_id: *order_id,
                        trader: order.trader.clone(),
                        filled_quantity: order.filled_quantity,
                        remaining_quantity: remaining,
                    });
                }
            }

            // Here we modify the StorageDoubleMap
            for (price, order_ids) in persistent_bids.iter() {
                if !order_ids.is_empty() {
                    match BoundedVec::<OrderId, T::MaxOrders>::try_from(order_ids.clone()) {
                        Ok(bounded) => {
                            Bids::<T>::insert(price, bounded);
                        }
                        Err(_) => {
                            // Doing this so that its save and bounded(altho this is mostly guaranteed because its from pending asks/bids and also pendingcancellations)
                            let truncated: Vec<OrderId> = order_ids
                                .iter()
                                .take(T::MaxOrders::get() as usize)
                                .cloned()
                                .collect();

                            if let Ok(bounded) =
                                BoundedVec::<OrderId, T::MaxOrders>::try_from(truncated.clone())
                            {
                                Bids::<T>::insert(price, bounded);
                            }
                        }
                    }
                }
            }

            for (price, order_ids) in persistent_asks.iter() {
                if !order_ids.is_empty() {
                    match BoundedVec::<OrderId, T::MaxOrders>::try_from(order_ids.clone()) {
                        Ok(bounded) => {
                            Asks::<T>::insert(price, bounded);
                        }
                        Err(_) => {
                            // Doing this so that its save and bounded(altho this is mostly guaranteed because its from pending asks/bids and also pendingcancellations)
                            let truncated: Vec<OrderId> = order_ids
                                .iter()
                                .take(T::MaxOrders::get() as usize)
                                .cloned()
                                .collect();

                            if let Ok(bounded) =
                                BoundedVec::<OrderId, T::MaxOrders>::try_from(truncated.clone())
                            {
                                Asks::<T>::insert(price, bounded);
                            }
                        }
                    }
                }
            }

            // Clear Pending Bids and Asks
            let _ = PendingBids::<T>::clear(u32::MAX, None);
            let _ = PendingAsks::<T>::clear(u32::MAX, None);

            //remove the cancellation storage
            PendingCancellations::<T>::kill();

            //EMIT event about complete trades
            Self::deposit_event(Event::MatchingCompleted {
                total_trades: all_trades.len() as u32,
                total_volume,
            });

            //Ok(())
        }
    }

    // ============================================================
    // EXTRINSICS
    // ============================================================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Place a limit order
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::place_order())]
        pub fn place_order(
            origin: OriginFor<T>,
            side: OrderSide,
            price: Amount,
            quantity: Amount,
            order_type: OrderType,
        ) -> DispatchResult {
            let trader = ensure_signed(origin)?;
            ensure!(price > 0, Error::<T>::InvalidPrice);
            ensure!(quantity > 0, Error::<T>::InvalidQuantity);

            let (asset, amount_to_lock) = match side {
                OrderSide::Buy => {
                    let total_amount = price
                        .checked_mul(quantity)
                        .ok_or(Error::<T>::ArithmeticOverflow)?;
                    (USDT, total_amount)
                }
                OrderSide::Sell => (ETH, quantity),
            };
            assets::Pallet::<T>::lock_funds(&trader, asset, amount_to_lock)?;

            let order_id = NextOrderId::<T>::get();
            let order = Order {
                order_id,
                trader: trader.clone(),
                side,
                status: OrderStatus::Open,
                order_type,
                price,
                quantity,
                filled_quantity: 0,
                ttl: None,
            };

            Orders::<T>::insert(order_id, order);
            if side == OrderSide::Buy {
                PendingBids::<T>::try_mutate(price, |orders| {
                    orders
                        .try_push(order_id)
                        .map_err(|_| Error::<T>::TooManyPendingOrders)
                })?;
            } else {
                PendingAsks::<T>::try_mutate(price, |orders| {
                    orders
                        .try_push(order_id)
                        .map_err(|_| Error::<T>::TooManyPendingOrders)
                })?;
            }

            UserOrders::<T>::try_mutate(trader.clone(), |orders| {
                orders
                    .try_push(order_id)
                    .map_err(|_| Error::<T>::TooManyUserOrders)
            })?;

            NextOrderId::<T>::put(order_id + 1);

            Self::deposit_event(Event::OrderPlaced {
                order_id,
                side,
                price,
                asset_id: if side == OrderSide::Buy { USDT } else { ETH },
                quantity,
            });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::cancel_order())]
        pub fn cancel_order(origin: OriginFor<T>, order_id: OrderId) -> DispatchResult {
            let trader = ensure_signed(origin)?;

            let order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;

            ensure!(trader == order.trader, Error::<T>::NotOrderOwner);
            ensure!(
                order.status != OrderStatus::Filled,
                Error::<T>::OrderNotActive
            );

            PendingCancellations::<T>::try_mutate(|cancellations| {
                cancellations
                    .try_push(order_id)
                    .map_err(|_| Error::<T>::TooManyPendingCancellations)
            })?;

            Self::deposit_event(Event::CancellationRequested {
                order_id: order.order_id,
                trader,
            });

            Ok(())
        }
    }

    // ======================================
    // Getter functions for storage/for some reason, directly acccessing them doesnt work
    // =======================================
    impl<T: Config> Pallet<T> {
        pub fn next_order_id() -> OrderId {
            NextOrderId::<T>::get()
        }

        /// Get the next trade ID
        pub fn next_trade_id() -> TradeId {
            NextTradeId::<T>::get()
        }

        /// Get an order by ID
        pub fn get_order(order_id: OrderId) -> Option<Order<T>> {
            Orders::<T>::get(order_id)
        }

        /// Get a trade by ID
        pub fn get_trade(trade_id: TradeId) -> Option<Trade<T>> {
            Trades::<T>::get(trade_id)
        }

        /// Get bids at a specific price level
        pub fn get_bids_at_price(price: Amount) -> Vec<OrderId> {
            Bids::<T>::get(price).into_inner()
        }

        /// Get asks at a specific price level
        pub fn get_asks_at_price(price: Amount) -> Vec<OrderId> {
            Asks::<T>::get(price).into_inner()
        }

        /// Get pending bids at a specific price level
        pub fn get_pending_bids_at_price(price: Amount) -> Vec<OrderId> {
            PendingBids::<T>::get(price).into_inner()
        }

        /// Get pending asks at a specific price level
        pub fn get_pending_asks_at_price(price: Amount) -> Vec<OrderId> {
            PendingAsks::<T>::get(price).into_inner()
        }

        /// Get pending cancellations
        pub fn get_pending_cancellations() -> Vec<OrderId> {
            PendingCancellations::<T>::get().into_inner()
        }

        /// Get user's orders
        pub fn get_user_orders(user: &T::AccountId) -> Vec<OrderId> {
            UserOrders::<T>::get(user).into_inner()
        }
    }
}
