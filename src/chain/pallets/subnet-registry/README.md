# Subnet Registry Pallet

A Substrate pallet for managing subnet definitions in the NeuroChain network.

## Overview

The Subnet Registry Pallet provides functionality for creating, updating, and managing subnets within the NeuroMesh decentralized network. Each subnet represents a distinct task domain with specific configuration parameters, schemas, and operational requirements.

## Features

- **Create Subnets**: Register new subnets with task-specific configurations
- **Update Subnets**: Modify subnet parameters (owner-only)
- **Retire Subnets**: Mark subnets as inactive to prevent new registrations
- **Query Subnets**: Retrieve subnet metadata and status information

## Storage Items

### Subnets
- **Type**: `StorageMap<u32, SubnetInfo<T>>`
- **Description**: Maps subnet IDs to their complete information structures

### NextSubnetId
- **Type**: `StorageValue<u32>`
- **Description**: Counter for generating unique subnet IDs

### SubnetCount
- **Type**: `StorageValue<u32>`
- **Description**: Total number of active subnets in the network

### OwnerSubnets
- **Type**: `StorageMap<AccountId, BoundedVec<u32>>`
- **Description**: Maps account IDs to their owned subnet IDs

## Data Structures

### SubnetInfo

```rust
pub struct SubnetInfo<T: Config> {
    pub id: u32,
    pub task_type: TaskType,
    pub input_schema: BoundedVec<u8, T::MaxSchemaSize>,
    pub output_schema: BoundedVec<u8, T::MaxSchemaSize>,
    pub evaluation_spec: BoundedVec<u8, T::MaxUriSize>,
    pub emission_weight: Percent,
    pub min_stake_miner: BalanceOf<T>,
    pub min_stake_validator: BalanceOf<T>,
    pub owner: T::AccountId,
    pub status: SubnetStatus,
}
```

### TaskType

Enumeration of supported task types:

- `CodeGen`: Code generation tasks (program synthesis, code completion)
- `ImageGen`: Image generation tasks (text-to-image, image editing)
- `ProteinFolding`: Protein folding and molecular structure prediction
- `Custom(BoundedVec<u8, 64>)`: Custom task type with string identifier

### SubnetStatus

- `Active`: Subnet is operational and accepting new registrations
- `Retired`: Subnet is inactive, no new registrations allowed

## Extrinsics

### create_subnet

Creates a new subnet with specified parameters.

**Parameters:**
- `origin`: Signed origin (becomes subnet owner)
- `task_type`: Classification of computational task
- `input_schema`: JSON schema for input validation
- `output_schema`: JSON schema for output validation
- `evaluation_spec`: URI to evaluation criteria
- `emission_weight`: Percentage of network emissions (0-100%)
- `min_stake_miner`: Minimum stake required for miners
- `min_stake_validator`: Minimum stake required for validators

**Requirements:**
- Caller must have sufficient balance for subnet deposit
- Emission weight must not exceed 100%
- Schema sizes must not exceed configured maximum
- Total subnet count must not exceed maximum limit

**Events:**
- `SubnetCreated { subnet_id, owner, task_type }`

**Errors:**
- `TooManySubnets`: Maximum subnet count reached
- `SchemaTooLarge`: Schema exceeds size limit
- `UriTooLarge`: URI exceeds size limit
- `InvalidEmissionWeight`: Weight exceeds 100%
- `InsufficientBalance`: Insufficient balance for deposit

### update_subnet

Updates an existing subnet's configuration.

**Parameters:**
- `origin`: Signed origin (must be subnet owner)
- `subnet_id`: ID of the subnet to update
- `input_schema`: Optional new input schema
- `output_schema`: Optional new output schema
- `evaluation_spec`: Optional new evaluation spec URI
- `emission_weight`: Optional new emission weight
- `min_stake_miner`: Optional new minimum stake for miners
- `min_stake_validator`: Optional new minimum stake for validators

**Requirements:**
- Caller must be the subnet owner
- Subnet must not be retired
- Updated parameters must meet the same validation as create_subnet

**Events:**
- `SubnetUpdated { subnet_id, owner }`

**Errors:**
- `SubnetNotFound`: Subnet doesn't exist
- `NotAuthorized`: Caller is not the owner
- `SubnetAlreadyRetired`: Cannot update retired subnet

### retire_subnet

Marks a subnet as retired to prevent new registrations.

**Parameters:**
- `origin`: Signed origin (must be subnet owner)
- `subnet_id`: ID of the subnet to retire

**Requirements:**
- Caller must be the subnet owner
- Subnet must not already be retired

**Events:**
- `SubnetRetired { subnet_id, owner }`

**Errors:**
- `SubnetNotFound`: Subnet doesn't exist
- `NotAuthorized`: Caller is not the owner
- `SubnetAlreadyRetired`: Subnet already retired

## Configuration

The pallet requires the following configuration parameters:

```rust
pub trait Config: frame_system::Config {
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

    #[pallet::constant]
    type MaxSchemaSize: Get<u32>;

    #[pallet::constant]
    type MaxUriSize: Get<u32>;

    #[pallet::constant]
    type MaxSubnets: Get<u32>;

    #[pallet::constant]
    type SubnetDeposit: Get<BalanceOf<Self>>;
}
```

### Recommended Configuration Values

- `MaxSchemaSize`: 10,000 bytes (sufficient for most JSON schemas)
- `MaxUriSize`: 1,000 bytes (supports IPFS and standard URIs)
- `MaxSubnets`: 100 (prevents storage bloat)
- `SubnetDeposit`: 1,000 units (anti-spam measure)

## Helper Functions

### subnet_exists

```rust
pub fn subnet_exists(subnet_id: u32) -> bool
```

Checks if a subnet exists.

### is_subnet_active

```rust
pub fn is_subnet_active(subnet_id: u32) -> bool
```

Checks if a subnet is active (exists and not retired).

### get_owner_subnet_count

```rust
pub fn get_owner_subnet_count(owner: &T::AccountId) -> u32
```

Returns the total number of subnets owned by an account.

## Usage Examples

### Creating a Subnet

```rust
// Create a code generation subnet
let task_type = TaskType::CodeGen;
let input_schema = br#"{"type": "object", "properties": {"prompt": {"type": "string"}}}"#.to_vec();
let output_schema = br#"{"type": "object", "properties": {"code": {"type": "string"}}}"#.to_vec();
let eval_spec = b"ipfs://QmExampleEvaluationSpec".to_vec();

SubnetRegistry::create_subnet(
    Origin::signed(account_id),
    task_type,
    input_schema,
    output_schema,
    eval_spec,
    Percent::from_percent(10),
    1_000_000,  // min_stake_miner
    2_000_000,  // min_stake_validator
)?;
```

### Updating a Subnet

```rust
// Update emission weight
SubnetRegistry::update_subnet(
    Origin::signed(owner),
    subnet_id,
    None,  // input_schema
    None,  // output_schema
    None,  // evaluation_spec
    Some(Percent::from_percent(15)),  // new emission weight
    None,  // min_stake_miner
    None,  // min_stake_validator
)?;
```

### Retiring a Subnet

```rust
SubnetRegistry::retire_subnet(
    Origin::signed(owner),
    subnet_id
)?;
```

## Testing

The pallet includes comprehensive unit tests covering:

- Subnet creation with various task types
- Deposit reservation and balance checks
- Parameter validation (emission weight, schema size, URI size)
- Authorization checks for updates and retirements
- Status transitions (active to retired)
- Multiple subnet management
- Owner-subnet relationship tracking

Run tests with:

```bash
cargo test
```

## Integration

To integrate this pallet into a Substrate runtime:

1. Add the pallet dependency to your runtime's `Cargo.toml`
2. Implement the `Config` trait for your runtime
3. Add the pallet to `construct_runtime!` macro
4. Set configuration parameters in runtime constants

Example runtime integration:

```rust
parameter_types! {
    pub const MaxSchemaSize: u32 = 10_000;
    pub const MaxUriSize: u32 = 1_000;
    pub const MaxSubnets: u32 = 100;
    pub const SubnetDeposit: Balance = 1_000 * UNITS;
}

impl pallet_subnet_registry::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MaxSchemaSize = MaxSchemaSize;
    type MaxUriSize = MaxUriSize;
    type MaxSubnets = MaxSubnets;
    type SubnetDeposit = SubnetDeposit;
}

construct_runtime! {
    pub enum Runtime {
        // ... other pallets
        SubnetRegistry: pallet_subnet_registry,
    }
}
```

## Security Considerations

1. **Deposit Requirement**: Subnet creation requires a deposit to prevent spam
2. **Owner Authorization**: Only subnet owners can update or retire their subnets
3. **Bounded Storage**: All variable-length data uses bounded vectors to prevent unbounded growth
4. **Immutable Retirement**: Retired subnets cannot be reactivated or updated
5. **Balance Checks**: Deposit reservation fails gracefully if insufficient balance

## License

MIT

## Contributing

Please ensure all code changes include appropriate tests and documentation.
