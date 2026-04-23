//! # Miner & Validator Registry Pallet
//!
//! Manages registration, UID allocation, endpoint metadata, and stake deposits
//! for miners and validators across NeuroChain subnets.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ReservableCurrency},
    };
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    pub type SubnetId = u32;
    pub type Uid = u32;

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum Role {
        Miner,
        Validator,
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct MinerInfo<T: Config> {
        pub uid: Uid,
        pub account: T::AccountId,
        pub endpoint: BoundedVec<u8, T::MaxEndpointLen>,
        pub stake: BalanceOf<T>,
        pub registered_at: BlockNumberFor<T>,
        pub unlock_at: Option<BlockNumberFor<T>>,
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct ValidatorInfo<T: Config> {
        pub uid: Uid,
        pub account: T::AccountId,
        pub endpoint: BoundedVec<u8, T::MaxEndpointLen>,
        pub stake: BalanceOf<T>,
        pub reputation: u32,
        pub registered_at: BlockNumberFor<T>,
        pub unlock_at: Option<BlockNumberFor<T>>,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        #[pallet::constant]
        type MaxEndpointLen: Get<u32>;

        #[pallet::constant]
        type UnlockPeriod: Get<BlockNumberFor<Self>>;

        #[pallet::constant]
        type MaxMinersPerSubnet: Get<u32>;

        #[pallet::constant]
        type MaxValidatorsPerSubnet: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Miner entries keyed by (subnet_id, uid).
    #[pallet::storage]
    #[pallet::getter(fn miners)]
    pub type Miners<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        SubnetId,
        Blake2_128Concat,
        Uid,
        MinerInfo<T>,
        OptionQuery,
    >;

    /// Monotonic UID counter per subnet for miner allocation.
    #[pallet::storage]
    #[pallet::getter(fn next_miner_uid)]
    pub type NextMinerUid<T: Config> = StorageMap<_, Blake2_128Concat, SubnetId, Uid, ValueQuery>;

    /// Active miner count per subnet.
    #[pallet::storage]
    #[pallet::getter(fn miner_count)]
    pub type MinerCount<T: Config> = StorageMap<_, Blake2_128Concat, SubnetId, u32, ValueQuery>;

    /// Account → (subnet_id, uid) lookup for miners.
    #[pallet::storage]
    #[pallet::getter(fn miner_of_account)]
    pub type MinerOfAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, (SubnetId, Uid), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn validators)]
    pub type Validators<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        SubnetId,
        Blake2_128Concat,
        Uid,
        ValidatorInfo<T>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn next_validator_uid)]
    pub type NextValidatorUid<T: Config> =
        StorageMap<_, Blake2_128Concat, SubnetId, Uid, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn validator_count)]
    pub type ValidatorCount<T: Config> = StorageMap<_, Blake2_128Concat, SubnetId, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn validator_of_account)]
    pub type ValidatorOfAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, (SubnetId, Uid), OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        MinerRegistered {
            subnet_id: SubnetId,
            uid: Uid,
            account: T::AccountId,
        },
        MinerEndpointUpdated {
            subnet_id: SubnetId,
            uid: Uid,
        },
        MinerDeregistered {
            subnet_id: SubnetId,
            uid: Uid,
            unlock_at: BlockNumberFor<T>,
        },
        MinerStakeWithdrawn {
            subnet_id: SubnetId,
            uid: Uid,
        },
        ValidatorRegistered {
            subnet_id: SubnetId,
            uid: Uid,
            account: T::AccountId,
        },
        ValidatorEndpointUpdated {
            subnet_id: SubnetId,
            uid: Uid,
        },
        ValidatorDeregistered {
            subnet_id: SubnetId,
            uid: Uid,
            unlock_at: BlockNumberFor<T>,
        },
        ValidatorStakeWithdrawn {
            subnet_id: SubnetId,
            uid: Uid,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        AlreadyRegistered,
        NotRegistered,
        NotOwner,
        SubnetFull,
        EndpointTooLong,
        InsufficientStake,
        StakeStillLocked,
        AlreadyDeregistered,
        ArithmeticOverflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register the caller as a miner on `subnet_id` with `stake` reserved and
        /// the given `endpoint` metadata. Allocates a fresh monotonic UID.
        pub fn register_miner(
            origin: OriginFor<T>,
            subnet_id: SubnetId,
            endpoint: Vec<u8>,
            stake: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                !MinerOfAccount::<T>::contains_key(&who),
                Error::<T>::AlreadyRegistered
            );

            let count = MinerCount::<T>::get(subnet_id);
            ensure!(count < T::MaxMinersPerSubnet::get(), Error::<T>::SubnetFull);

            let endpoint_bounded: BoundedVec<u8, T::MaxEndpointLen> = endpoint
                .try_into()
                .map_err(|_| Error::<T>::EndpointTooLong)?;

            T::Currency::reserve(&who, stake).map_err(|_| Error::<T>::InsufficientStake)?;

            let uid = NextMinerUid::<T>::get(subnet_id);
            let next_uid = uid.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
            NextMinerUid::<T>::insert(subnet_id, next_uid);

            let now = <frame_system::Pallet<T>>::block_number();
            let info = MinerInfo {
                uid,
                account: who.clone(),
                endpoint: endpoint_bounded,
                stake,
                registered_at: now,
                unlock_at: None,
            };

            Miners::<T>::insert(subnet_id, uid, info);
            MinerOfAccount::<T>::insert(&who, (subnet_id, uid));
            MinerCount::<T>::insert(
                subnet_id,
                count.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?,
            );

            Self::deposit_event(Event::MinerRegistered {
                subnet_id,
                uid,
                account: who,
            });
            Ok(())
        }

        /// Update the miner's endpoint metadata.
        pub fn update_miner_endpoint(
            origin: OriginFor<T>,
            subnet_id: SubnetId,
            uid: Uid,
            endpoint: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Miners::<T>::try_mutate(subnet_id, uid, |maybe| {
                let info = maybe.as_mut().ok_or(Error::<T>::NotRegistered)?;
                ensure!(info.account == who, Error::<T>::NotOwner);
                info.endpoint = endpoint
                    .try_into()
                    .map_err(|_| Error::<T>::EndpointTooLong)?;
                Self::deposit_event(Event::MinerEndpointUpdated { subnet_id, uid });
                Ok(())
            })
        }

        /// Begin deregistration. Stake remains reserved until `UnlockPeriod` passes,
        /// then `withdraw_miner_stake` can release it.
        pub fn deregister_miner(
            origin: OriginFor<T>,
            subnet_id: SubnetId,
            uid: Uid,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Miners::<T>::try_mutate(subnet_id, uid, |maybe| {
                let info = maybe.as_mut().ok_or(Error::<T>::NotRegistered)?;
                ensure!(info.account == who, Error::<T>::NotOwner);
                ensure!(info.unlock_at.is_none(), Error::<T>::AlreadyDeregistered);
                let unlock_at = <frame_system::Pallet<T>>::block_number() + T::UnlockPeriod::get();
                info.unlock_at = Some(unlock_at);
                Self::deposit_event(Event::MinerDeregistered {
                    subnet_id,
                    uid,
                    unlock_at,
                });
                Ok(())
            })
        }

        /// Release the reserved stake once the unlock block has been reached and
        /// remove the miner entry.
        pub fn withdraw_miner_stake(
            origin: OriginFor<T>,
            subnet_id: SubnetId,
            uid: Uid,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let info = Miners::<T>::get(subnet_id, uid).ok_or(Error::<T>::NotRegistered)?;
            ensure!(info.account == who, Error::<T>::NotOwner);
            let unlock = info.unlock_at.ok_or(Error::<T>::AlreadyRegistered)?;
            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(now >= unlock, Error::<T>::StakeStillLocked);

            T::Currency::unreserve(&who, info.stake);
            Miners::<T>::remove(subnet_id, uid);
            MinerOfAccount::<T>::remove(&who);
            MinerCount::<T>::mutate(subnet_id, |c| *c = c.saturating_sub(1));
            Self::deposit_event(Event::MinerStakeWithdrawn { subnet_id, uid });
            Ok(())
        }

        pub fn register_validator(
            origin: OriginFor<T>,
            subnet_id: SubnetId,
            endpoint: Vec<u8>,
            stake: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                !ValidatorOfAccount::<T>::contains_key(&who),
                Error::<T>::AlreadyRegistered
            );

            let count = ValidatorCount::<T>::get(subnet_id);
            ensure!(
                count < T::MaxValidatorsPerSubnet::get(),
                Error::<T>::SubnetFull
            );

            let endpoint_bounded: BoundedVec<u8, T::MaxEndpointLen> = endpoint
                .try_into()
                .map_err(|_| Error::<T>::EndpointTooLong)?;

            T::Currency::reserve(&who, stake).map_err(|_| Error::<T>::InsufficientStake)?;

            let uid = NextValidatorUid::<T>::get(subnet_id);
            let next_uid = uid.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
            NextValidatorUid::<T>::insert(subnet_id, next_uid);

            let now = <frame_system::Pallet<T>>::block_number();
            let info = ValidatorInfo {
                uid,
                account: who.clone(),
                endpoint: endpoint_bounded,
                stake,
                reputation: 0,
                registered_at: now,
                unlock_at: None,
            };

            Validators::<T>::insert(subnet_id, uid, info);
            ValidatorOfAccount::<T>::insert(&who, (subnet_id, uid));
            ValidatorCount::<T>::insert(
                subnet_id,
                count.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?,
            );

            Self::deposit_event(Event::ValidatorRegistered {
                subnet_id,
                uid,
                account: who,
            });
            Ok(())
        }

        pub fn update_validator_endpoint(
            origin: OriginFor<T>,
            subnet_id: SubnetId,
            uid: Uid,
            endpoint: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Validators::<T>::try_mutate(subnet_id, uid, |maybe| {
                let info = maybe.as_mut().ok_or(Error::<T>::NotRegistered)?;
                ensure!(info.account == who, Error::<T>::NotOwner);
                info.endpoint = endpoint
                    .try_into()
                    .map_err(|_| Error::<T>::EndpointTooLong)?;
                Self::deposit_event(Event::ValidatorEndpointUpdated { subnet_id, uid });
                Ok(())
            })
        }

        pub fn deregister_validator(
            origin: OriginFor<T>,
            subnet_id: SubnetId,
            uid: Uid,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Validators::<T>::try_mutate(subnet_id, uid, |maybe| {
                let info = maybe.as_mut().ok_or(Error::<T>::NotRegistered)?;
                ensure!(info.account == who, Error::<T>::NotOwner);
                ensure!(info.unlock_at.is_none(), Error::<T>::AlreadyDeregistered);
                let unlock_at = <frame_system::Pallet<T>>::block_number() + T::UnlockPeriod::get();
                info.unlock_at = Some(unlock_at);
                Self::deposit_event(Event::ValidatorDeregistered {
                    subnet_id,
                    uid,
                    unlock_at,
                });
                Ok(())
            })
        }

        pub fn withdraw_validator_stake(
            origin: OriginFor<T>,
            subnet_id: SubnetId,
            uid: Uid,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let info = Validators::<T>::get(subnet_id, uid).ok_or(Error::<T>::NotRegistered)?;
            ensure!(info.account == who, Error::<T>::NotOwner);
            let unlock = info.unlock_at.ok_or(Error::<T>::AlreadyRegistered)?;
            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(now >= unlock, Error::<T>::StakeStillLocked);

            T::Currency::unreserve(&who, info.stake);
            Validators::<T>::remove(subnet_id, uid);
            ValidatorOfAccount::<T>::remove(&who);
            ValidatorCount::<T>::mutate(subnet_id, |c| *c = c.saturating_sub(1));
            Self::deposit_event(Event::ValidatorStakeWithdrawn { subnet_id, uid });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn is_miner(account: &T::AccountId) -> bool {
            MinerOfAccount::<T>::contains_key(account)
        }
        pub fn is_validator(account: &T::AccountId) -> bool {
            ValidatorOfAccount::<T>::contains_key(account)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as pallet_registry;
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
            Registry: pallet_registry,
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
        type AccountData = pallet_balances::AccountData<u64>;
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
        pub const ExistentialDeposit: u64 = 1;
    }

    impl pallet_balances::Config for Test {
        type MaxLocks = ();
        type MaxReserves = ();
        type ReserveIdentifier = [u8; 8];
        type Balance = u64;
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
        pub const MaxEndpointLen: u32 = 128;
        pub const UnlockPeriod: u64 = 10;
        pub const MaxMinersPerSubnet: u32 = 100;
        pub const MaxValidatorsPerSubnet: u32 = 50;
    }

    impl Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type Currency = Balances;
        type MaxEndpointLen = MaxEndpointLen;
        type UnlockPeriod = UnlockPeriod;
        type MaxMinersPerSubnet = MaxMinersPerSubnet;
        type MaxValidatorsPerSubnet = MaxValidatorsPerSubnet;
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap();
        pallet_balances::GenesisConfig::<Test> {
            balances: vec![(1, 100_000), (2, 100_000), (3, 100_000)],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        t.into()
    }

    #[test]
    fn miner_register_allocates_monotonic_uid() {
        new_test_ext().execute_with(|| {
            assert_ok!(Registry::register_miner(
                RuntimeOrigin::signed(1),
                1,
                b"http://a".to_vec(),
                1_000
            ));
            assert_ok!(Registry::register_miner(
                RuntimeOrigin::signed(2),
                1,
                b"http://b".to_vec(),
                1_000
            ));
            assert_eq!(Registry::miner_of_account(1).unwrap(), (1, 0));
            assert_eq!(Registry::miner_of_account(2).unwrap(), (1, 1));
            assert_eq!(Registry::miner_count(1), 2);
            assert_eq!(Registry::next_miner_uid(1), 2);
        });
    }

    #[test]
    fn miner_double_register_fails() {
        new_test_ext().execute_with(|| {
            assert_ok!(Registry::register_miner(
                RuntimeOrigin::signed(1),
                1,
                b"http://a".to_vec(),
                1_000
            ));
            assert_noop!(
                Registry::register_miner(RuntimeOrigin::signed(1), 1, b"http://a2".to_vec(), 1_000),
                Error::<Test>::AlreadyRegistered
            );
        });
    }

    #[test]
    fn miner_register_reserves_stake() {
        new_test_ext().execute_with(|| {
            let before = Balances::free_balance(1);
            assert_ok!(Registry::register_miner(
                RuntimeOrigin::signed(1),
                1,
                b"http://a".to_vec(),
                5_000
            ));
            assert_eq!(Balances::free_balance(1), before - 5_000);
            assert_eq!(Balances::reserved_balance(1), 5_000);
        });
    }

    #[test]
    fn miner_update_endpoint_works() {
        new_test_ext().execute_with(|| {
            assert_ok!(Registry::register_miner(
                RuntimeOrigin::signed(1),
                1,
                b"http://a".to_vec(),
                1_000
            ));
            assert_ok!(Registry::update_miner_endpoint(
                RuntimeOrigin::signed(1),
                1,
                0,
                b"http://new".to_vec()
            ));
            let m = Registry::miners(1, 0).unwrap();
            assert_eq!(m.endpoint.as_slice(), b"http://new");
        });
    }

    #[test]
    fn miner_update_not_owner_fails() {
        new_test_ext().execute_with(|| {
            assert_ok!(Registry::register_miner(
                RuntimeOrigin::signed(1),
                1,
                b"http://a".to_vec(),
                1_000
            ));
            assert_noop!(
                Registry::update_miner_endpoint(RuntimeOrigin::signed(2), 1, 0, b"x".to_vec()),
                Error::<Test>::NotOwner
            );
        });
    }

    #[test]
    fn miner_deregister_and_withdraw_flow() {
        new_test_ext().execute_with(|| {
            assert_ok!(Registry::register_miner(
                RuntimeOrigin::signed(1),
                1,
                b"http://a".to_vec(),
                1_000
            ));
            assert_ok!(Registry::deregister_miner(RuntimeOrigin::signed(1), 1, 0));
            // Before unlock period passes
            assert_noop!(
                Registry::withdraw_miner_stake(RuntimeOrigin::signed(1), 1, 0),
                Error::<Test>::StakeStillLocked
            );

            System::set_block_number(System::block_number() + UnlockPeriod::get());

            assert_ok!(Registry::withdraw_miner_stake(
                RuntimeOrigin::signed(1),
                1,
                0
            ));
            assert!(Registry::miners(1, 0).is_none());
            assert_eq!(Balances::reserved_balance(1), 0);
            assert_eq!(Registry::miner_count(1), 0);
        });
    }

    #[test]
    fn miner_double_deregister_fails() {
        new_test_ext().execute_with(|| {
            assert_ok!(Registry::register_miner(
                RuntimeOrigin::signed(1),
                1,
                b"http://a".to_vec(),
                1_000
            ));
            assert_ok!(Registry::deregister_miner(RuntimeOrigin::signed(1), 1, 0));
            assert_noop!(
                Registry::deregister_miner(RuntimeOrigin::signed(1), 1, 0),
                Error::<Test>::AlreadyDeregistered
            );
        });
    }

    #[test]
    fn validator_register_allocates_monotonic_uid() {
        new_test_ext().execute_with(|| {
            assert_ok!(Registry::register_validator(
                RuntimeOrigin::signed(1),
                1,
                b"ws://a".to_vec(),
                1_000
            ));
            assert_ok!(Registry::register_validator(
                RuntimeOrigin::signed(2),
                1,
                b"ws://b".to_vec(),
                1_000
            ));
            assert_eq!(Registry::validator_of_account(1).unwrap(), (1, 0));
            assert_eq!(Registry::validator_of_account(2).unwrap(), (1, 1));
            assert_eq!(Registry::validator_count(1), 2);
        });
    }

    #[test]
    fn validator_deregister_and_withdraw_flow() {
        new_test_ext().execute_with(|| {
            assert_ok!(Registry::register_validator(
                RuntimeOrigin::signed(1),
                1,
                b"ws://a".to_vec(),
                1_000
            ));
            assert_ok!(Registry::deregister_validator(
                RuntimeOrigin::signed(1),
                1,
                0
            ));
            System::set_block_number(System::block_number() + UnlockPeriod::get());
            assert_ok!(Registry::withdraw_validator_stake(
                RuntimeOrigin::signed(1),
                1,
                0
            ));
            assert!(Registry::validators(1, 0).is_none());
        });
    }

    #[test]
    fn endpoint_too_long_fails() {
        new_test_ext().execute_with(|| {
            let big = vec![b'x'; (MaxEndpointLen::get() + 1) as usize];
            assert_noop!(
                Registry::register_miner(RuntimeOrigin::signed(1), 1, big, 1_000),
                Error::<Test>::EndpointTooLong
            );
        });
    }

    #[test]
    fn insufficient_stake_fails() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                Registry::register_miner(RuntimeOrigin::signed(99), 1, b"x".to_vec(), 1_000),
                Error::<Test>::InsufficientStake
            );
        });
    }

    #[test]
    fn miner_role_isolated_from_validator() {
        new_test_ext().execute_with(|| {
            assert_ok!(Registry::register_miner(
                RuntimeOrigin::signed(1),
                1,
                b"m".to_vec(),
                1_000
            ));
            // Same account can still register as validator (different role)
            assert_ok!(Registry::register_validator(
                RuntimeOrigin::signed(1),
                1,
                b"v".to_vec(),
                1_000
            ));
            assert!(Registry::is_miner(&1));
            assert!(Registry::is_validator(&1));
        });
    }
}
