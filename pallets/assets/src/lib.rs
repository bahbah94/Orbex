//ensures it compiles to wasm
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

pub const USDT: u32 = 0;
pub const ETH: u32 = 1;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
    use frame_system::pallet_prelude::{OriginFor, *};

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // UserID -> Token -> Value
    #[pallet::storage]
    pub type FreeBalance<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        u32,
        u128,
        ValueQuery,
    >;

    //same with locked balance
    // userid --> token --> value
    #[pallet::storage]
    pub type LockedBalance<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        u32,
        u128,
        ValueQuery,
    >;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Deposited {
            user: T::AccountId,
            asset_id: u32,
            amount: u128,
        },
        Withdrawn {
            user: T::AccountId,
            asset_id: u32,
            amount: u128,
        },
        Locked {
            user: T::AccountId,
            asset_id: u32,
            amount: u128,
        },
        Unlocked {
            user: T::AccountId,
            asset_id: u32,
            amount: u128,
        },
        Transferred {
            from: T::AccountId,
            to: T::AccountId,
            asset_id: u32,
            amount: u128,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        InsufficientFreeBalance,
        InsufficientLockedBalance,
        InvalidAsset,
        AmountZero,
    }

    //Now we write the extrinsincs deposit, withdraw, lock and unlock & also transfer
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::deposit())]
        pub fn deposit(origin: OriginFor<T>, asset_id: u32, amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(amount > 0, Error::<T>::AmountZero);
            ensure!(asset_id == 0 || asset_id == 1, Error::<T>::InvalidAsset);

            FreeBalance::<T>::mutate(who.clone(), asset_id, |balance| {
                *balance = balance.saturating_add(amount);
            });

            Self::deposit_event(Event::Deposited {
                user: who,
                asset_id,
                amount,
            });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::withdraw())]
        pub fn withdraw(origin: OriginFor<T>, asset_id: u32, amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(amount > 0, Error::<T>::AmountZero);
            ensure!(
                asset_id == USDT || asset_id == ETH,
                Error::<T>::InvalidAsset
            );

            FreeBalance::<T>::try_mutate(who.clone(), asset_id, |balance| {
                ensure!(*balance >= amount, Error::<T>::InsufficientFreeBalance);
                *balance = balance.saturating_sub(amount);
                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::Withdrawn {
                user: who,
                asset_id,
                amount,
            });
            Ok(())
        }
    }
    impl<T: Config> Pallet<T> {
        /// Get free balance (helper for tests)
        pub fn get_free_balance(user: &T::AccountId, asset_id: u32) -> u128 {
            FreeBalance::<T>::get(user, asset_id)
        }

        /// Get locked balance (helper for tests)
        pub fn get_locked_balance(user: &T::AccountId, asset_id: u32) -> u128 {
            LockedBalance::<T>::get(user, asset_id)
        }
        // locking funds for when trading happens, user cannot simply just withdraw stuff
        pub fn lock_funds(user: &T::AccountId, asset_id: u32, amount: u128) -> DispatchResult {
            // we move from freebalance and shift to lockedbalance
            FreeBalance::<T>::try_mutate(user, asset_id, |balance| {
                ensure!(*balance >= amount, Error::<T>::InsufficientFreeBalance);
                *balance = balance.saturating_sub(amount);
                Ok::<_, DispatchError>(())
            })?;

            LockedBalance::<T>::mutate(user, asset_id, |balance| {
                *balance = balance.saturating_add(amount);
            });

            Self::deposit_event(Event::Locked {
                user: (*user).clone(),
                asset_id,
                amount,
            });

            Ok(())
        }

        // This can happen when we say cancel and order
        pub fn unlock_funds(user: &T::AccountId, asset_id: u32, amount: u128) -> DispatchResult {
            // Move from locked to free
            LockedBalance::<T>::try_mutate(user, asset_id, |locked| {
                ensure!(*locked >= amount, Error::<T>::InsufficientLockedBalance);
                *locked = locked.saturating_sub(amount);
                Ok::<_, DispatchError>(())
            })?;

            FreeBalance::<T>::mutate(user, asset_id, |balance| {
                *balance = balance.saturating_add(amount);
            });

            Self::deposit_event(Event::Unlocked {
                user: user.clone(),
                asset_id,
                amount,
            });

            Ok(())
        }

        pub fn transfer_locked(
            from: &T::AccountId,
            to: &T::AccountId,
            asset_id: u32,
            amount: u128,
        ) -> DispatchResult {
            // remove from transferee
            LockedBalance::<T>::try_mutate(from, asset_id, |balance| {
                ensure!(*balance >= amount, Error::<T>::InsufficientLockedBalance);
                *balance = balance.saturating_sub(amount);
                Ok::<_, DispatchError>(())
            })?;

            //move it to transferred .ie to account
            FreeBalance::<T>::mutate(to, asset_id, |balance| {
                *balance = balance.saturating_add(amount)
            });

            Self::deposit_event(Event::Transferred {
                from: from.clone(),
                to: to.clone(),
                asset_id,
                amount,
            });

            Ok(())
        }
    }
}
