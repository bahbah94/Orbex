use super::*;

#[allow(unused)]
use crate::Pallet as Assets;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

mod benchmark{
    use super::*;

    #[benchmark]
    fn deposit(){
        let caller : T::AccountId = whitelisted_caller();
        let asset_id: u32 = 0u32;
        let amount: u128 = 1000u128;

        #[extrinsic_call]
        deposit(RawOrigin::Signed(caller.clone()), asset_id, amount);

        assert_eq!(
            FreeBalance::<T>::get(&caller, asset_id),
            amount
        );
    }

    #[benchmark]
    fn withdraw(){
            let caller: T::AccountId = whitelisted_caller();
            let asset_id = 0u32;
            let amount = 1000u128;

            //fake deposit
            FreeBalance::<T>::insert(&caller, asset_id, amount);

            //now withdraw
            #[extrinsic_call]
            withdraw(RawOrigin::Signed(caller), asset_id, amount);

            assert_eq!(
                FreeBalance::<T>::get(&caller, asset_id),
                0
            );
    }

    impl_benchmark_test_suite!(Asset, crate::mock::new_test_ext(, crate::mock::Test));
}