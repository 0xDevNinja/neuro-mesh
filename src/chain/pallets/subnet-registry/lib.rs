//! # Subnet Registry Pallet
//!
//! The Subnet Registry Pallet provides functionality for managing subnet definitions
//! in the NeuroChain network. Each subnet represents a distinct task domain with
//! specific configuration parameters, schemas, and operational requirements.
//!
//! ## Overview
//!
//! This pallet enables:
//! - Creation of new subnets with specific task types and parameters
//! - Update of subnet configurations by authorized owners
//! - Retirement of subnets to prevent new registrations
//! - Querying subnet metadata and status
//!
//! ## Terminology
//!
//! - **Subnet**: A logical partition of the network for a specific task type
//! - **Task Type**: Category of computational work (CODE_GEN, IMAGE_GEN, etc.)
//! - **Schema**: JSON schema defining input/output data structures
//! - **Evaluation Spec**: URI pointing to evaluation criteria and methodology
//! - **Emission Weight**: Percentage of total network emissions allocated to this subnet
//! - **Staking Threshold**: Minimum stake required for miners and validators
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `create_subnet` - Create a new subnet with specified parameters
//! - `update_subnet` - Update an existing subnet's configuration
//! - `retire_subnet` - Mark a subnet as retired to prevent new registrations

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ReservableCurrency},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::Percent;
    use sp_std::vec::Vec;

    /// Type alias for substrate balance type
    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// Task type enumeration for subnet classification
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum TaskType {
        /// Code generation tasks (e.g., program synthesis, code completion)
        CodeGen,
        /// Image generation tasks (e.g., text-to-image, image editing)
        ImageGen,
        /// Protein folding and molecular structure prediction
        ProteinFolding,
        /// Custom task type with string identifier (bounded length)
        Custom(BoundedVec<u8, ConstU32<64>>),
    }

    /// Subnet status enumeration
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum SubnetStatus {
        /// Subnet is active and accepting registrations
        Active,
        /// Subnet is retired, no new registrations allowed
        Retired,
    }

    /// Subnet information structure
    ///
    /// Contains all metadata and configuration parameters for a subnet
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct SubnetInfo<T: Config> {
        /// Unique subnet identifier
        pub id: u32,
        /// Task type classification
        pub task_type: TaskType,
        /// JSON schema for input validation (stored as bounded vector)
        pub input_schema: BoundedVec<u8, T::MaxSchemaSize>,
        /// JSON schema for output validation (stored as bounded vector)
        pub output_schema: BoundedVec<u8, T::MaxSchemaSize>,
        /// URI to evaluation specification document
        pub evaluation_spec: BoundedVec<u8, T::MaxUriSize>,
        /// Percentage of network emissions allocated to this subnet
        pub emission_weight: Percent,
        /// Minimum stake required for miners
        pub min_stake_miner: BalanceOf<T>,
        /// Minimum stake required for validators
        pub min_stake_validator: BalanceOf<T>,
        /// Owner account with update privileges
        pub owner: T::AccountId,
        /// Current operational status
        pub status: SubnetStatus,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Currency type for staking operations
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// Maximum size of schema data in bytes
        #[pallet::constant]
        type MaxSchemaSize: Get<u32>;

        /// Maximum size of URI in bytes
        #[pallet::constant]
        type MaxUriSize: Get<u32>;

        /// Maximum number of subnets that can exist
        #[pallet::constant]
        type MaxSubnets: Get<u32>;

        /// Deposit required to create a subnet
        #[pallet::constant]
        type SubnetDeposit: Get<BalanceOf<Self>>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Storage for subnet information by subnet ID
    #[pallet::storage]
    #[pallet::getter(fn subnets)]
    pub type Subnets<T: Config> =
        StorageMap<_, Blake2_128Concat, u32, SubnetInfo<T>, OptionQuery>;

    /// Counter for the next available subnet ID
    #[pallet::storage]
    #[pallet::getter(fn next_subnet_id)]
    pub type NextSubnetId<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Total count of active subnets
    #[pallet::storage]
    #[pallet::getter(fn subnet_count)]
    pub type SubnetCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Mapping from owner to their subnet IDs
    #[pallet::storage]
    #[pallet::getter(fn owner_subnets)]
    pub type OwnerSubnets<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<u32, T::MaxSubnets>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new subnet was created
        /// [subnet_id, owner, task_type]
        SubnetCreated {
            subnet_id: u32,
            owner: T::AccountId,
            task_type: TaskType,
        },
        /// A subnet was updated
        /// [subnet_id, owner]
        SubnetUpdated {
            subnet_id: u32,
            owner: T::AccountId,
        },
        /// A subnet was retired
        /// [subnet_id, owner]
        SubnetRetired {
            subnet_id: u32,
            owner: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Subnet does not exist
        SubnetNotFound,
        /// Not authorized to perform this action
        NotAuthorized,
        /// Subnet is already retired
        SubnetAlreadyRetired,
        /// Maximum number of subnets reached
        TooManySubnets,
        /// Schema size exceeds maximum allowed
        SchemaTooLarge,
        /// URI size exceeds maximum allowed
        UriTooLarge,
        /// Invalid emission weight (must be 0-100%)
        InvalidEmissionWeight,
        /// Owner has too many subnets
        TooManyOwnedSubnets,
        /// Arithmetic overflow occurred
        ArithmeticOverflow,
        /// Insufficient balance for deposit
        InsufficientBalance,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new subnet with specified parameters
        ///
        /// # Parameters
        ///
        /// - `origin`: Must be signed, will become the subnet owner
        /// - `task_type`: Classification of computational task
        /// - `input_schema`: JSON schema for input validation
        /// - `output_schema`: JSON schema for output validation
        /// - `evaluation_spec`: URI to evaluation criteria
        /// - `emission_weight`: Percentage of network emissions (0-100)
        /// - `min_stake_miner`: Minimum stake for miners
        /// - `min_stake_validator`: Minimum stake for validators
        ///
        /// # Events
        ///
        /// Emits `SubnetCreated` on success
        ///
        /// # Errors
        ///
        /// - `TooManySubnets` if maximum subnet count reached
        /// - `SchemaTooLarge` if schema exceeds size limit
        /// - `UriTooLarge` if URI exceeds size limit
        /// - `InvalidEmissionWeight` if weight > 100%
        /// - `TooManyOwnedSubnets` if owner has too many subnets
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn create_subnet(
            origin: OriginFor<T>,
            task_type: TaskType,
            input_schema: Vec<u8>,
            output_schema: Vec<u8>,
            evaluation_spec: Vec<u8>,
            emission_weight: Percent,
            min_stake_miner: BalanceOf<T>,
            min_stake_validator: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Validate parameters
            ensure!(
                emission_weight <= Percent::from_percent(100),
                Error::<T>::InvalidEmissionWeight
            );

            // Check subnet count limit
            let current_count = SubnetCount::<T>::get();
            ensure!(
                current_count < T::MaxSubnets::get(),
                Error::<T>::TooManySubnets
            );

            // Convert to bounded vectors
            let input_schema_bounded: BoundedVec<u8, T::MaxSchemaSize> = input_schema
                .try_into()
                .map_err(|_| Error::<T>::SchemaTooLarge)?;
            let output_schema_bounded: BoundedVec<u8, T::MaxSchemaSize> = output_schema
                .try_into()
                .map_err(|_| Error::<T>::SchemaTooLarge)?;
            let evaluation_spec_bounded: BoundedVec<u8, T::MaxUriSize> = evaluation_spec
                .try_into()
                .map_err(|_| Error::<T>::UriTooLarge)?;

            // Reserve deposit
            T::Currency::reserve(&who, T::SubnetDeposit::get())
                .map_err(|_| Error::<T>::InsufficientBalance)?;

            // Get next subnet ID
            let subnet_id = NextSubnetId::<T>::get();
            let next_id = subnet_id
                .checked_add(1)
                .ok_or(Error::<T>::ArithmeticOverflow)?;

            // Create subnet info
            let subnet_info = SubnetInfo {
                id: subnet_id,
                task_type: task_type.clone(),
                input_schema: input_schema_bounded,
                output_schema: output_schema_bounded,
                evaluation_spec: evaluation_spec_bounded,
                emission_weight,
                min_stake_miner,
                min_stake_validator,
                owner: who.clone(),
                status: SubnetStatus::Active,
            };

            // Store subnet
            Subnets::<T>::insert(subnet_id, subnet_info);

            // Update owner's subnet list
            OwnerSubnets::<T>::try_mutate(&who, |subnets| {
                subnets
                    .try_push(subnet_id)
                    .map_err(|_| Error::<T>::TooManyOwnedSubnets)
            })?;

            // Update counters
            NextSubnetId::<T>::put(next_id);
            SubnetCount::<T>::put(
                current_count
                    .checked_add(1)
                    .ok_or(Error::<T>::ArithmeticOverflow)?,
            );

            // Emit event
            Self::deposit_event(Event::SubnetCreated {
                subnet_id,
                owner: who,
                task_type,
            });

            Ok(())
        }

        /// Update an existing subnet's configuration
        ///
        /// Only the subnet owner can update the subnet configuration.
        /// Cannot update a retired subnet.
        ///
        /// # Parameters
        ///
        /// - `origin`: Must be signed by the subnet owner
        /// - `subnet_id`: ID of the subnet to update
        /// - `input_schema`: New input schema (optional)
        /// - `output_schema`: New output schema (optional)
        /// - `evaluation_spec`: New evaluation spec URI (optional)
        /// - `emission_weight`: New emission weight (optional)
        /// - `min_stake_miner`: New minimum stake for miners (optional)
        /// - `min_stake_validator`: New minimum stake for validators (optional)
        ///
        /// # Events
        ///
        /// Emits `SubnetUpdated` on success
        ///
        /// # Errors
        ///
        /// - `SubnetNotFound` if subnet doesn't exist
        /// - `NotAuthorized` if caller is not the owner
        /// - `SubnetAlreadyRetired` if subnet is retired
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn update_subnet(
            origin: OriginFor<T>,
            subnet_id: u32,
            input_schema: Option<Vec<u8>>,
            output_schema: Option<Vec<u8>>,
            evaluation_spec: Option<Vec<u8>>,
            emission_weight: Option<Percent>,
            min_stake_miner: Option<BalanceOf<T>>,
            min_stake_validator: Option<BalanceOf<T>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Get subnet and verify ownership
            Subnets::<T>::try_mutate(subnet_id, |maybe_subnet| {
                let subnet = maybe_subnet.as_mut().ok_or(Error::<T>::SubnetNotFound)?;

                ensure!(subnet.owner == who, Error::<T>::NotAuthorized);
                ensure!(
                    subnet.status != SubnetStatus::Retired,
                    Error::<T>::SubnetAlreadyRetired
                );

                // Update fields if provided
                if let Some(schema) = input_schema {
                    subnet.input_schema = schema
                        .try_into()
                        .map_err(|_| Error::<T>::SchemaTooLarge)?;
                }

                if let Some(schema) = output_schema {
                    subnet.output_schema = schema
                        .try_into()
                        .map_err(|_| Error::<T>::SchemaTooLarge)?;
                }

                if let Some(spec) = evaluation_spec {
                    subnet.evaluation_spec =
                        spec.try_into().map_err(|_| Error::<T>::UriTooLarge)?;
                }

                if let Some(weight) = emission_weight {
                    ensure!(
                        weight <= Percent::from_percent(100),
                        Error::<T>::InvalidEmissionWeight
                    );
                    subnet.emission_weight = weight;
                }

                if let Some(stake) = min_stake_miner {
                    subnet.min_stake_miner = stake;
                }

                if let Some(stake) = min_stake_validator {
                    subnet.min_stake_validator = stake;
                }

                // Emit event
                Self::deposit_event(Event::SubnetUpdated {
                    subnet_id,
                    owner: who,
                });

                Ok(())
            })
        }

        /// Retire a subnet to prevent new registrations
        ///
        /// Only the subnet owner can retire their subnet.
        /// Retired subnets remain in storage but cannot accept new miners/validators.
        ///
        /// # Parameters
        ///
        /// - `origin`: Must be signed by the subnet owner
        /// - `subnet_id`: ID of the subnet to retire
        ///
        /// # Events
        ///
        /// Emits `SubnetRetired` on success
        ///
        /// # Errors
        ///
        /// - `SubnetNotFound` if subnet doesn't exist
        /// - `NotAuthorized` if caller is not the owner
        /// - `SubnetAlreadyRetired` if subnet is already retired
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn retire_subnet(origin: OriginFor<T>, subnet_id: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Get subnet and verify ownership
            Subnets::<T>::try_mutate(subnet_id, |maybe_subnet| {
                let subnet = maybe_subnet.as_mut().ok_or(Error::<T>::SubnetNotFound)?;

                ensure!(subnet.owner == who, Error::<T>::NotAuthorized);
                ensure!(
                    subnet.status != SubnetStatus::Retired,
                    Error::<T>::SubnetAlreadyRetired
                );

                // Update status
                subnet.status = SubnetStatus::Retired;

                // Emit event
                Self::deposit_event(Event::SubnetRetired {
                    subnet_id,
                    owner: who,
                });

                Ok(())
            })
        }
    }

    impl<T: Config> Pallet<T> {
        /// Check if a subnet exists
        pub fn subnet_exists(subnet_id: u32) -> bool {
            Subnets::<T>::contains_key(subnet_id)
        }

        /// Check if a subnet is active
        pub fn is_subnet_active(subnet_id: u32) -> bool {
            Subnets::<T>::get(subnet_id)
                .map(|s| s.status == SubnetStatus::Active)
                .unwrap_or(false)
        }

        /// Get the total number of subnets owned by an account
        pub fn get_owner_subnet_count(owner: &T::AccountId) -> u32 {
            OwnerSubnets::<T>::get(owner).len() as u32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as pallet_subnet_registry;
    use frame_support::{
        assert_noop, assert_ok, parameter_types,
        traits::{ConstU32, ConstU64},
    };
    use sp_core::H256;
    use sp_runtime::{
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage, Percent,
    };

    type Block = frame_system::mocking::MockBlock<Test>;

    // Configure a mock runtime for testing
    frame_support::construct_runtime!(
        pub enum Test {
            System: frame_system,
            Balances: pallet_balances,
            SubnetRegistry: pallet_subnet_registry,
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
        pub const MaxSchemaSize: u32 = 10_000;
        pub const MaxUriSize: u32 = 1_000;
        pub const MaxSubnets: u32 = 100;
        pub const SubnetDeposit: u64 = 1000;
    }

    impl Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type Currency = Balances;
        type MaxSchemaSize = MaxSchemaSize;
        type MaxUriSize = MaxUriSize;
        type MaxSubnets = MaxSubnets;
        type SubnetDeposit = SubnetDeposit;
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap();

        pallet_balances::GenesisConfig::<Test> {
            balances: vec![(1, 100000), (2, 100000), (3, 100000)],
        }
        .assimilate_storage(&mut t)
        .unwrap();

        t.into()
    }

    #[test]
    fn create_subnet_works() {
        new_test_ext().execute_with(|| {
            let owner = 1u64;
            let task_type = TaskType::CodeGen;
            let input_schema = b"{}".to_vec();
            let output_schema = b"{}".to_vec();
            let eval_spec = b"ipfs://QmExample".to_vec();
            let emission_weight = Percent::from_percent(10);

            assert_ok!(SubnetRegistry::create_subnet(
                RuntimeOrigin::signed(owner),
                task_type.clone(),
                input_schema,
                output_schema,
                eval_spec,
                emission_weight,
                1000,
                2000,
            ));

            // Verify subnet was created
            assert_eq!(SubnetRegistry::next_subnet_id(), 1);
            assert_eq!(SubnetRegistry::subnet_count(), 1);

            let subnet = SubnetRegistry::subnets(0).unwrap();
            assert_eq!(subnet.id, 0);
            assert_eq!(subnet.task_type, TaskType::CodeGen);
            assert_eq!(subnet.owner, owner);
            assert_eq!(subnet.status, SubnetStatus::Active);
            assert_eq!(subnet.emission_weight, emission_weight);

            // Verify owner mapping
            let owner_subnets = SubnetRegistry::owner_subnets(owner);
            assert_eq!(owner_subnets.len(), 1);
            assert_eq!(owner_subnets[0], 0);
        });
    }

    #[test]
    fn create_subnet_reserves_deposit() {
        new_test_ext().execute_with(|| {
            let owner = 1u64;
            let initial_balance = Balances::free_balance(owner);

            assert_ok!(SubnetRegistry::create_subnet(
                RuntimeOrigin::signed(owner),
                TaskType::ImageGen,
                b"{}".to_vec(),
                b"{}".to_vec(),
                b"ipfs://QmExample".to_vec(),
                Percent::from_percent(5),
                1000,
                2000,
            ));

            let final_balance = Balances::free_balance(owner);
            assert_eq!(
                initial_balance - final_balance,
                SubnetDeposit::get()
            );
        });
    }

    #[test]
    fn create_subnet_fails_with_invalid_emission_weight() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                SubnetRegistry::create_subnet(
                    RuntimeOrigin::signed(1),
                    TaskType::CodeGen,
                    b"{}".to_vec(),
                    b"{}".to_vec(),
                    b"ipfs://QmExample".to_vec(),
                    Percent::from_percent(101),
                    1000,
                    2000,
                ),
                Error::<Test>::InvalidEmissionWeight
            );
        });
    }

    #[test]
    fn create_subnet_fails_with_schema_too_large() {
        new_test_ext().execute_with(|| {
            let large_schema = vec![0u8; (MaxSchemaSize::get() + 1) as usize];

            assert_noop!(
                SubnetRegistry::create_subnet(
                    RuntimeOrigin::signed(1),
                    TaskType::CodeGen,
                    large_schema,
                    b"{}".to_vec(),
                    b"ipfs://QmExample".to_vec(),
                    Percent::from_percent(10),
                    1000,
                    2000,
                ),
                Error::<Test>::SchemaTooLarge
            );
        });
    }

    #[test]
    fn create_subnet_fails_with_insufficient_balance() {
        new_test_ext().execute_with(|| {
            // Account with insufficient balance
            assert_noop!(
                SubnetRegistry::create_subnet(
                    RuntimeOrigin::signed(99),
                    TaskType::CodeGen,
                    b"{}".to_vec(),
                    b"{}".to_vec(),
                    b"ipfs://QmExample".to_vec(),
                    Percent::from_percent(10),
                    1000,
                    2000,
                ),
                Error::<Test>::InsufficientBalance
            );
        });
    }

    #[test]
    fn update_subnet_works() {
        new_test_ext().execute_with(|| {
            let owner = 1u64;

            // Create subnet first
            assert_ok!(SubnetRegistry::create_subnet(
                RuntimeOrigin::signed(owner),
                TaskType::CodeGen,
                b"{}".to_vec(),
                b"{}".to_vec(),
                b"ipfs://QmExample".to_vec(),
                Percent::from_percent(10),
                1000,
                2000,
            ));

            // Update emission weight
            let new_weight = Percent::from_percent(20);
            assert_ok!(SubnetRegistry::update_subnet(
                RuntimeOrigin::signed(owner),
                0,
                None,
                None,
                None,
                Some(new_weight),
                None,
                None,
            ));

            let subnet = SubnetRegistry::subnets(0).unwrap();
            assert_eq!(subnet.emission_weight, new_weight);
        });
    }

    #[test]
    fn update_subnet_fails_if_not_owner() {
        new_test_ext().execute_with(|| {
            // Create subnet with owner 1
            assert_ok!(SubnetRegistry::create_subnet(
                RuntimeOrigin::signed(1),
                TaskType::CodeGen,
                b"{}".to_vec(),
                b"{}".to_vec(),
                b"ipfs://QmExample".to_vec(),
                Percent::from_percent(10),
                1000,
                2000,
            ));

            // Try to update as account 2
            assert_noop!(
                SubnetRegistry::update_subnet(
                    RuntimeOrigin::signed(2),
                    0,
                    None,
                    None,
                    None,
                    Some(Percent::from_percent(20)),
                    None,
                    None,
                ),
                Error::<Test>::NotAuthorized
            );
        });
    }

    #[test]
    fn update_subnet_fails_if_retired() {
        new_test_ext().execute_with(|| {
            let owner = 1u64;

            // Create and retire subnet
            assert_ok!(SubnetRegistry::create_subnet(
                RuntimeOrigin::signed(owner),
                TaskType::CodeGen,
                b"{}".to_vec(),
                b"{}".to_vec(),
                b"ipfs://QmExample".to_vec(),
                Percent::from_percent(10),
                1000,
                2000,
            ));

            assert_ok!(SubnetRegistry::retire_subnet(
                RuntimeOrigin::signed(owner),
                0
            ));

            // Try to update retired subnet
            assert_noop!(
                SubnetRegistry::update_subnet(
                    RuntimeOrigin::signed(owner),
                    0,
                    None,
                    None,
                    None,
                    Some(Percent::from_percent(20)),
                    None,
                    None,
                ),
                Error::<Test>::SubnetAlreadyRetired
            );
        });
    }

    #[test]
    fn retire_subnet_works() {
        new_test_ext().execute_with(|| {
            let owner = 1u64;

            // Create subnet
            assert_ok!(SubnetRegistry::create_subnet(
                RuntimeOrigin::signed(owner),
                TaskType::CodeGen,
                b"{}".to_vec(),
                b"{}".to_vec(),
                b"ipfs://QmExample".to_vec(),
                Percent::from_percent(10),
                1000,
                2000,
            ));

            // Retire subnet
            assert_ok!(SubnetRegistry::retire_subnet(
                RuntimeOrigin::signed(owner),
                0
            ));

            let subnet = SubnetRegistry::subnets(0).unwrap();
            assert_eq!(subnet.status, SubnetStatus::Retired);
            assert!(!SubnetRegistry::is_subnet_active(0));
        });
    }

    #[test]
    fn retire_subnet_fails_if_not_owner() {
        new_test_ext().execute_with(|| {
            // Create subnet with owner 1
            assert_ok!(SubnetRegistry::create_subnet(
                RuntimeOrigin::signed(1),
                TaskType::CodeGen,
                b"{}".to_vec(),
                b"{}".to_vec(),
                b"ipfs://QmExample".to_vec(),
                Percent::from_percent(10),
                1000,
                2000,
            ));

            // Try to retire as account 2
            assert_noop!(
                SubnetRegistry::retire_subnet(RuntimeOrigin::signed(2), 0),
                Error::<Test>::NotAuthorized
            );
        });
    }

    #[test]
    fn retire_subnet_fails_if_already_retired() {
        new_test_ext().execute_with(|| {
            let owner = 1u64;

            // Create and retire subnet
            assert_ok!(SubnetRegistry::create_subnet(
                RuntimeOrigin::signed(owner),
                TaskType::CodeGen,
                b"{}".to_vec(),
                b"{}".to_vec(),
                b"ipfs://QmExample".to_vec(),
                Percent::from_percent(10),
                1000,
                2000,
            ));

            assert_ok!(SubnetRegistry::retire_subnet(
                RuntimeOrigin::signed(owner),
                0
            ));

            // Try to retire again
            assert_noop!(
                SubnetRegistry::retire_subnet(RuntimeOrigin::signed(owner), 0),
                Error::<Test>::SubnetAlreadyRetired
            );
        });
    }

    #[test]
    fn subnet_exists_works() {
        new_test_ext().execute_with(|| {
            assert!(!SubnetRegistry::subnet_exists(0));

            assert_ok!(SubnetRegistry::create_subnet(
                RuntimeOrigin::signed(1),
                TaskType::CodeGen,
                b"{}".to_vec(),
                b"{}".to_vec(),
                b"ipfs://QmExample".to_vec(),
                Percent::from_percent(10),
                1000,
                2000,
            ));

            assert!(SubnetRegistry::subnet_exists(0));
            assert!(!SubnetRegistry::subnet_exists(1));
        });
    }

    #[test]
    fn multiple_subnets_can_be_created() {
        new_test_ext().execute_with(|| {
            // Create multiple subnets with different task types
            assert_ok!(SubnetRegistry::create_subnet(
                RuntimeOrigin::signed(1),
                TaskType::CodeGen,
                b"{}".to_vec(),
                b"{}".to_vec(),
                b"ipfs://QmExample1".to_vec(),
                Percent::from_percent(10),
                1000,
                2000,
            ));

            assert_ok!(SubnetRegistry::create_subnet(
                RuntimeOrigin::signed(2),
                TaskType::ImageGen,
                b"{}".to_vec(),
                b"{}".to_vec(),
                b"ipfs://QmExample2".to_vec(),
                Percent::from_percent(15),
                1500,
                2500,
            ));

            assert_ok!(SubnetRegistry::create_subnet(
                RuntimeOrigin::signed(1),
                TaskType::ProteinFolding,
                b"{}".to_vec(),
                b"{}".to_vec(),
                b"ipfs://QmExample3".to_vec(),
                Percent::from_percent(20),
                2000,
                3000,
            ));

            assert_eq!(SubnetRegistry::subnet_count(), 3);
            assert_eq!(SubnetRegistry::next_subnet_id(), 3);

            // Verify owner 1 has 2 subnets
            assert_eq!(SubnetRegistry::get_owner_subnet_count(&1), 2);
            // Verify owner 2 has 1 subnet
            assert_eq!(SubnetRegistry::get_owner_subnet_count(&2), 1);
        });
    }

    #[test]
    fn custom_task_type_works() {
        new_test_ext().execute_with(|| {
            let custom_type = TaskType::Custom(
                b"AUDIO_TRANSCRIPTION"
                    .to_vec()
                    .try_into()
                    .expect("bounded vec creation"),
            );

            assert_ok!(SubnetRegistry::create_subnet(
                RuntimeOrigin::signed(1),
                custom_type.clone(),
                b"{}".to_vec(),
                b"{}".to_vec(),
                b"ipfs://QmExample".to_vec(),
                Percent::from_percent(10),
                1000,
                2000,
            ));

            let subnet = SubnetRegistry::subnets(0).unwrap();
            assert_eq!(subnet.task_type, custom_type);
        });
    }
}
