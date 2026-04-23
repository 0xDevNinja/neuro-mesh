//! # Emissions Pallet
//!
//! Provides the emission schedule, per-subnet allocation, and per-epoch miner
//! and validator reward accounting for NeuroChain (CORE-004).
//!
//! Rewards are ledgered as `Balance` amounts that downstream payout logic can
//! mint/transfer. This pallet intentionally does not mint — it only computes
//! and records entitlements, which keeps it testable in isolation and lets the
//! runtime wire minting/transfer via its own currency trait.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::Currency};
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    pub type SubnetId = u32;
    pub type Uid = u32;
    pub type EpochIndex = u64;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Currency: Currency<Self::AccountId>;

        /// Total tokens emitted per epoch across the whole network.
        #[pallet::constant]
        type EpochEmission: Get<BalanceOf<Self>>;

        /// Fraction (ppm) of each subnet's emission awarded to validators.
        /// The remainder goes to miners. Default policy: 18% validators.
        #[pallet::constant]
        type ValidatorSharePpm: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Subnet emission weight (ppm of total network emission). Sum over all
    /// active subnets SHOULD be <= 1_000_000; this pallet enforces that invariant
    /// only when `set_subnet_weight` is called.
    #[pallet::storage]
    #[pallet::getter(fn subnet_weight)]
    pub type SubnetWeight<T: Config> = StorageMap<_, Blake2_128Concat, SubnetId, u32, ValueQuery>;

    /// Running sum of subnet weights, kept in sync with `SubnetWeight`.
    #[pallet::storage]
    #[pallet::getter(fn total_subnet_weight)]
    pub type TotalSubnetWeight<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Accrued but unclaimed rewards per account.
    #[pallet::storage]
    #[pallet::getter(fn pending_reward)]
    pub type PendingReward<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        SubnetWeightSet {
            subnet_id: SubnetId,
            weight_ppm: u32,
            total_ppm: u32,
        },
        EpochRewardsDistributed {
            subnet_id: SubnetId,
            epoch: EpochIndex,
            subnet_emission: BalanceOf<T>,
            miner_share: BalanceOf<T>,
            validator_share: BalanceOf<T>,
        },
        RewardClaimed {
            account: T::AccountId,
            amount: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidWeight,
        TotalWeightOverflow,
        NothingToClaim,
        SumsMismatch,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Set the emission weight (ppm) for a subnet. Rejected if the new
        /// total would exceed 1_000_000 ppm.
        pub fn set_subnet_weight(
            origin: OriginFor<T>,
            subnet_id: SubnetId,
            weight_ppm: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(weight_ppm <= 1_000_000, Error::<T>::InvalidWeight);

            let previous = SubnetWeight::<T>::get(subnet_id);
            let total = TotalSubnetWeight::<T>::get();
            let new_total = total
                .checked_sub(previous)
                .ok_or(Error::<T>::TotalWeightOverflow)?
                .checked_add(weight_ppm)
                .ok_or(Error::<T>::TotalWeightOverflow)?;
            ensure!(new_total <= 1_000_000, Error::<T>::InvalidWeight);

            SubnetWeight::<T>::insert(subnet_id, weight_ppm);
            TotalSubnetWeight::<T>::put(new_total);
            Self::deposit_event(Event::SubnetWeightSet {
                subnet_id,
                weight_ppm,
                total_ppm: new_total,
            });
            Ok(())
        }

        /// Distribute one epoch's emission for a given subnet.
        ///
        /// `miner_shares`: per-miner weight shares (normalized or raw — we
        /// normalize by sum here).
        /// `validator_reputations`: per-validator reputation values.
        ///
        /// Rewards are accrued into `PendingReward` and can later be claimed
        /// via `claim_rewards`.
        pub fn distribute_epoch_rewards(
            origin: OriginFor<T>,
            subnet_id: SubnetId,
            epoch: EpochIndex,
            miner_shares: Vec<(T::AccountId, u64)>,
            validator_reputations: Vec<(T::AccountId, u64)>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            // subnet_weight_ppm is absolute: 250_000 == 25% of the epoch pool.
            // Unallocated weight (1_000_000 - sum) simply does not get minted,
            // which mirrors deflationary / unallocated-supply semantics.
            let subnet_w = SubnetWeight::<T>::get(subnet_id) as u128;
            let epoch_emission: u128 = Self::balance_to_u128(T::EpochEmission::get());
            let subnet_emission = epoch_emission.saturating_mul(subnet_w) / 1_000_000;

            let validator_share_ppm = T::ValidatorSharePpm::get() as u128;
            let validator_emission =
                subnet_emission.saturating_mul(validator_share_ppm) / 1_000_000;
            let miner_emission = subnet_emission.saturating_sub(validator_emission);

            let miner_sum: u128 = miner_shares.iter().map(|(_, s)| *s as u128).sum();
            let val_sum: u128 = validator_reputations.iter().map(|(_, r)| *r as u128).sum();

            let mut miner_distributed: u128 = 0;
            if miner_sum > 0 {
                for (acc, s) in &miner_shares {
                    let reward = miner_emission.saturating_mul(*s as u128) / miner_sum;
                    miner_distributed = miner_distributed.saturating_add(reward);
                    Self::credit(acc, Self::u128_to_balance(reward));
                }
            }
            let mut val_distributed: u128 = 0;
            if val_sum > 0 {
                for (acc, r) in &validator_reputations {
                    let reward = validator_emission.saturating_mul(*r as u128) / val_sum;
                    val_distributed = val_distributed.saturating_add(reward);
                    Self::credit(acc, Self::u128_to_balance(reward));
                }
            }

            // Sanity: never overpay (only rounding dust should go unspent).
            ensure!(
                miner_distributed <= miner_emission && val_distributed <= validator_emission,
                Error::<T>::SumsMismatch
            );

            Self::deposit_event(Event::EpochRewardsDistributed {
                subnet_id,
                epoch,
                subnet_emission: Self::u128_to_balance(subnet_emission),
                miner_share: Self::u128_to_balance(miner_distributed),
                validator_share: Self::u128_to_balance(val_distributed),
            });
            Ok(())
        }

        /// Claim accrued rewards. This pallet does not mint — the runtime is
        /// expected to hold a pre-funded account, or downstream logic wires in
        /// a mint-on-claim via Currency::deposit_creating.
        pub fn claim_rewards(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let amount = PendingReward::<T>::take(&who);
            ensure!(amount > Zero::zero(), Error::<T>::NothingToClaim);
            // `deposit_creating` lets this pallet stand alone in tests; in a
            // real runtime the treasury would transfer instead.
            let _ = T::Currency::deposit_creating(&who, amount);
            Self::deposit_event(Event::RewardClaimed {
                account: who,
                amount,
            });
            Ok(())
        }
    }

    use sp_runtime::traits::{Saturating, Zero};

    impl<T: Config> Pallet<T> {
        fn credit(who: &T::AccountId, amount: BalanceOf<T>) {
            if amount > Zero::zero() {
                PendingReward::<T>::mutate(who, |b| *b = b.saturating_add(amount));
            }
        }

        fn balance_to_u128(b: BalanceOf<T>) -> u128 {
            b.try_into().ok().unwrap_or(u128::MAX)
        }

        fn u128_to_balance(v: u128) -> BalanceOf<T> {
            v.try_into().unwrap_or_else(|_| Zero::zero())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as pallet_emissions;
    use frame_support::{assert_noop, assert_ok, parameter_types, traits::ConstU32};
    use sp_core::H256;
    use sp_runtime::{
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    };

    type Block = frame_system::mocking::MockBlock<Test>;

    frame_support::construct_runtime!(
        pub enum Test {
            System: frame_system,
            Balances: pallet_balances,
            Emissions: pallet_emissions,
        }
    );

    parameter_types! {
        pub const BlockHashCount: u64 = 250;
    }

    impl frame_system::Config for Test {
        type BaseCallFilter = frame_support::traits::Everything;
        type BlockWeights = ();
        type BlockLength = ();
        type DbWeight = ();
        type RuntimeOrigin = RuntimeOrigin;
        type RuntimeCall = RuntimeCall;
        type Nonce = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Block = Block;
        type RuntimeEvent = RuntimeEvent;
        type RuntimeTask = RuntimeTask;
        type BlockHashCount = BlockHashCount;
        type Version = ();
        type PalletInfo = PalletInfo;
        type AccountData = pallet_balances::AccountData<u128>;
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
        type SS58Prefix = ();
        type OnSetCode = ();
        type MaxConsumers = ConstU32<16>;
        type SingleBlockMigrations = ();
        type MultiBlockMigrator = ();
        type PreInherents = ();
        type PostInherents = ();
        type PostTransactions = ();
    }

    parameter_types! {
        pub const ExistentialDeposit: u128 = 1;
    }

    impl pallet_balances::Config for Test {
        type MaxLocks = ();
        type MaxReserves = ();
        type ReserveIdentifier = [u8; 8];
        type Balance = u128;
        type RuntimeEvent = RuntimeEvent;
        type DustRemoval = ();
        type ExistentialDeposit = ExistentialDeposit;
        type AccountStore = System;
        type WeightInfo = ();
        type FreezeIdentifier = ();
        type MaxFreezes = ();
        type RuntimeHoldReason = ();
        type RuntimeFreezeReason = ();
    }

    parameter_types! {
        pub const EpochEmission: u128 = 1_000_000;
        pub const ValidatorSharePpm: u32 = 180_000;
    }

    impl Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type Currency = Balances;
        type EpochEmission = EpochEmission;
        type ValidatorSharePpm = ValidatorSharePpm;
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap()
            .into()
    }

    #[test]
    fn set_subnet_weight_enforces_ppm_cap() {
        new_test_ext().execute_with(|| {
            assert_ok!(Emissions::set_subnet_weight(
                RuntimeOrigin::root(),
                1,
                500_000
            ));
            assert_ok!(Emissions::set_subnet_weight(
                RuntimeOrigin::root(),
                2,
                500_000
            ));
            assert_noop!(
                Emissions::set_subnet_weight(RuntimeOrigin::root(), 3, 1),
                Error::<Test>::InvalidWeight
            );
            assert_eq!(Emissions::total_subnet_weight(), 1_000_000);
        });
    }

    #[test]
    fn set_subnet_weight_overwrites_correctly() {
        new_test_ext().execute_with(|| {
            assert_ok!(Emissions::set_subnet_weight(
                RuntimeOrigin::root(),
                1,
                500_000
            ));
            assert_ok!(Emissions::set_subnet_weight(
                RuntimeOrigin::root(),
                1,
                200_000
            ));
            assert_eq!(Emissions::total_subnet_weight(), 200_000);
        });
    }

    #[test]
    fn distribute_splits_by_validator_share() {
        new_test_ext().execute_with(|| {
            // Only subnet 1, weight 100%
            assert_ok!(Emissions::set_subnet_weight(
                RuntimeOrigin::root(),
                1,
                1_000_000
            ));
            // Two miners with equal shares, one validator gets full validator pool.
            assert_ok!(Emissions::distribute_epoch_rewards(
                RuntimeOrigin::root(),
                1,
                0,
                vec![(10, 1), (11, 1)],
                vec![(20, 1)]
            ));
            let miner_emission = 1_000_000u128 - 180_000; // 82% miners
            let each_miner = miner_emission / 2;
            assert_eq!(Emissions::pending_reward(10), each_miner);
            assert_eq!(Emissions::pending_reward(11), each_miner);
            assert_eq!(Emissions::pending_reward(20), 180_000u128);
        });
    }

    #[test]
    fn distribute_scales_by_subnet_weight() {
        new_test_ext().execute_with(|| {
            assert_ok!(Emissions::set_subnet_weight(
                RuntimeOrigin::root(),
                1,
                250_000 // 25% of network
            ));
            assert_ok!(Emissions::distribute_epoch_rewards(
                RuntimeOrigin::root(),
                1,
                0,
                vec![(10, 1)],
                vec![]
            ));
            // Subnet gets 25% of 1_000_000 = 250_000; miner share = 82% = 205_000
            assert_eq!(Emissions::pending_reward(10), 205_000u128);
        });
    }

    #[test]
    fn distribute_handles_empty_validator_list() {
        new_test_ext().execute_with(|| {
            assert_ok!(Emissions::set_subnet_weight(
                RuntimeOrigin::root(),
                1,
                1_000_000
            ));
            assert_ok!(Emissions::distribute_epoch_rewards(
                RuntimeOrigin::root(),
                1,
                0,
                vec![(10, 1)],
                vec![]
            ));
            // Miner gets 82% of 1_000_000
            assert_eq!(Emissions::pending_reward(10), 820_000u128);
        });
    }

    #[test]
    fn claim_rewards_mints_and_clears() {
        new_test_ext().execute_with(|| {
            assert_ok!(Emissions::set_subnet_weight(
                RuntimeOrigin::root(),
                1,
                1_000_000
            ));
            assert_ok!(Emissions::distribute_epoch_rewards(
                RuntimeOrigin::root(),
                1,
                0,
                vec![(10, 1)],
                vec![]
            ));
            assert_ok!(Emissions::claim_rewards(RuntimeOrigin::signed(10)));
            assert_eq!(Emissions::pending_reward(10), 0);
            assert_eq!(Balances::free_balance(10), 820_000u128);
        });
    }

    #[test]
    fn claim_rewards_errors_when_empty() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                Emissions::claim_rewards(RuntimeOrigin::signed(10)),
                Error::<Test>::NothingToClaim
            );
        });
    }

    #[test]
    fn reward_weights_proportional_to_share() {
        new_test_ext().execute_with(|| {
            assert_ok!(Emissions::set_subnet_weight(
                RuntimeOrigin::root(),
                1,
                1_000_000
            ));
            assert_ok!(Emissions::distribute_epoch_rewards(
                RuntimeOrigin::root(),
                1,
                0,
                vec![(10, 1), (11, 3)],
                vec![]
            ));
            let miner_pool = 820_000u128;
            assert_eq!(Emissions::pending_reward(10), miner_pool / 4);
            assert_eq!(Emissions::pending_reward(11), miner_pool * 3 / 4);
        });
    }

    #[test]
    fn invalid_weight_rejected() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                Emissions::set_subnet_weight(RuntimeOrigin::root(), 1, 1_000_001),
                Error::<Test>::InvalidWeight
            );
        });
    }
}
