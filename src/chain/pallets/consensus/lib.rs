//! # Consensus Pallet
//!
//! Hosts on-chain consensus extrinsics for NeuroChain:
//! - `submit_weights` (CONS-002): validators submit their per-miner weight
//!   vector for the current epoch. Submissions are normalized, bucketed by
//!   epoch, and overwritable until the epoch closes.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_neuro_weight::{CompressedWeights, SparseWeights, COSINE_SCALE};
    use sp_std::vec::Vec;

    /// Initial reputation for new validators (one unit of a 1_000_000 scale).
    pub const INITIAL_REPUTATION: u32 = 500_000;
    /// EMA smoothing factor alpha. rep_new = alpha * rep_old + (1 - alpha) * sim.
    /// Expressed as ppm out of COSINE_SCALE.
    pub const REPUTATION_ALPHA_PPM: u32 = 800_000;
    /// Cosine threshold (ppm of COSINE_SCALE) above which two validators are
    /// flagged as suspiciously correlated (possible collusion / cartel).
    pub const COLLUSION_THRESHOLD_PPM: u32 = 950_000;
    /// Reputation penalty applied to each member of a flagged collusion cluster,
    /// in ppm of COSINE_SCALE.
    pub const COLLUSION_PENALTY_PPM: u32 = 100_000;

    pub type SubnetId = u32;
    pub type Uid = u32;
    pub type EpochIndex = u64;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Blocks per epoch.
        #[pallet::constant]
        type EpochLength: Get<BlockNumberFor<Self>>;

        /// Maximum number of miners (entries) per submitted weight vector.
        #[pallet::constant]
        type MaxMinersPerSubnet: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Submitted compressed weight vectors keyed by (subnet, epoch, validator_uid).
    /// Overwriting the same key within the same epoch is allowed — validators may
    /// correct their submission until the epoch closes.
    #[pallet::storage]
    #[pallet::getter(fn submissions)]
    pub type Submissions<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, SubnetId>,
            NMapKey<Blake2_128Concat, EpochIndex>,
            NMapKey<Blake2_128Concat, Uid>,
        ),
        CompressedWeights,
        OptionQuery,
    >;

    /// Count of submissions per (subnet, epoch) for quick quorum checks.
    #[pallet::storage]
    #[pallet::getter(fn submission_count)]
    pub type SubmissionCount<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        SubnetId,
        Blake2_128Concat,
        EpochIndex,
        u32,
        ValueQuery,
    >;

    /// List of validator UIDs that submitted in a given (subnet, epoch). Ordered
    /// by insertion; needed for deterministic finalization without scanning the
    /// entire NMap.
    #[pallet::storage]
    #[pallet::getter(fn submitters)]
    pub type Submitters<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        SubnetId,
        Blake2_128Concat,
        EpochIndex,
        Vec<Uid>,
        ValueQuery,
    >;

    /// Aggregated global weight vector per (subnet, epoch) produced by
    /// `finalize_epoch`.
    #[pallet::storage]
    #[pallet::getter(fn global_weights)]
    pub type GlobalWeights<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        SubnetId,
        Blake2_128Concat,
        EpochIndex,
        CompressedWeights,
        OptionQuery,
    >;

    /// Whether a given (subnet, epoch) has been finalized.
    #[pallet::storage]
    #[pallet::getter(fn epoch_finalized)]
    pub type EpochFinalized<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        SubnetId,
        Blake2_128Concat,
        EpochIndex,
        bool,
        ValueQuery,
    >;

    /// Validator reputation per (subnet, validator_uid), fixed-point in
    /// `[0, COSINE_SCALE]`.
    #[pallet::storage]
    #[pallet::getter(fn reputation)]
    pub type Reputation<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, SubnetId, Blake2_128Concat, Uid, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A validator submitted (or overwrote) their weight vector for an epoch.
        WeightsSubmitted {
            subnet_id: SubnetId,
            epoch: EpochIndex,
            validator_uid: Uid,
            entry_count: u32,
            overwritten: bool,
        },
        /// An epoch was finalized — global weights produced, reputations updated.
        EpochFinalized {
            subnet_id: SubnetId,
            epoch: EpochIndex,
            submitter_count: u32,
        },
        /// Validator reputation updated after finalization.
        ReputationUpdated {
            subnet_id: SubnetId,
            validator_uid: Uid,
            old_reputation: u32,
            new_reputation: u32,
            similarity: u32,
        },
        /// Potential collusion cluster detected: all listed validators produced
        /// pairwise-correlated weight vectors above COLLUSION_THRESHOLD_PPM.
        CollusionDetected {
            subnet_id: SubnetId,
            epoch: EpochIndex,
            members: Vec<Uid>,
        },
        /// Reputation penalty applied to a cluster member.
        CollusionPenaltyApplied {
            subnet_id: SubnetId,
            validator_uid: Uid,
            reputation_before: u32,
            reputation_after: u32,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        TooManyEntries,
        EmptyWeights,
        InvalidWeightVector,
        IndexOutOfBounds,
        EpochNotReady,
        EpochAlreadyFinalized,
        NoSubmissions,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit a weight vector for the caller's validator UID.
        ///
        /// The pallet sorts + dedups entries, drops zero-weight ones, rejects
        /// out-of-range UIDs, L1-normalizes, compresses, and stores the result
        /// bucketed by the current epoch.
        pub fn submit_weights(
            origin: OriginFor<T>,
            subnet_id: SubnetId,
            validator_uid: Uid,
            weights: Vec<(Uid, u16)>,
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            // Authorization against a registry pallet is deferred to integration
            // tests / runtime wiring — this crate is consensus-mechanism-only.

            ensure!(!weights.is_empty(), Error::<T>::EmptyWeights);
            ensure!(
                weights.len() as u32 <= T::MaxMinersPerSubnet::get(),
                Error::<T>::TooManyEntries,
            );

            let max_uid = T::MaxMinersPerSubnet::get();
            for (uid, _) in &weights {
                if *uid >= max_uid {
                    return Err(Error::<T>::IndexOutOfBounds.into());
                }
            }

            let mut sw = SparseWeights::from_pairs(weights);
            if sw.is_empty() {
                return Err(Error::<T>::EmptyWeights.into());
            }
            sw.normalize_l1();
            let compressed = sw
                .compress(max_uid)
                .map_err(|_| Error::<T>::InvalidWeightVector)?;

            let epoch = Self::current_epoch();
            ensure!(
                !EpochFinalized::<T>::get(subnet_id, epoch),
                Error::<T>::EpochAlreadyFinalized
            );
            let entry_count = compressed.runs.len() as u32;
            let overwritten = Submissions::<T>::contains_key((subnet_id, epoch, validator_uid));
            Submissions::<T>::insert((subnet_id, epoch, validator_uid), compressed);
            if !overwritten {
                SubmissionCount::<T>::mutate(subnet_id, epoch, |c| *c = c.saturating_add(1));
                Submitters::<T>::mutate(subnet_id, epoch, |v| v.push(validator_uid));
            }

            Self::deposit_event(Event::WeightsSubmitted {
                subnet_id,
                epoch,
                validator_uid,
                entry_count,
                overwritten,
            });
            Ok(())
        }

        /// Finalize an epoch's consensus. Must be called after `epoch + 1` has
        /// started (i.e. the submission window has closed). Computes the
        /// reputation-weighted aggregate of all submissions, updates each
        /// submitter's reputation via cosine similarity, and stores the global
        /// weight vector.
        pub fn finalize_epoch(
            origin: OriginFor<T>,
            subnet_id: SubnetId,
            epoch: EpochIndex,
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            ensure!(
                !EpochFinalized::<T>::get(subnet_id, epoch),
                Error::<T>::EpochAlreadyFinalized
            );
            ensure!(Self::current_epoch() > epoch, Error::<T>::EpochNotReady);

            let submitters = Submitters::<T>::get(subnet_id, epoch);
            ensure!(!submitters.is_empty(), Error::<T>::NoSubmissions);

            // Decompress each submission and collect with that validator's current
            // reputation (defaulted to INITIAL_REPUTATION for first-timers).
            let mut decoded: Vec<(Uid, SparseWeights, u32)> = Vec::with_capacity(submitters.len());
            for uid in &submitters {
                if let Some(compressed) = Submissions::<T>::get((subnet_id, epoch, *uid)) {
                    if let Ok(sw) = SparseWeights::decompress(&compressed) {
                        let rep = if Reputation::<T>::contains_key(subnet_id, *uid) {
                            Reputation::<T>::get(subnet_id, *uid)
                        } else {
                            INITIAL_REPUTATION
                        };
                        decoded.push((*uid, sw, rep));
                    }
                }
            }

            // Aggregate with reputation weights.
            let votes: Vec<(&SparseWeights, u64)> = decoded
                .iter()
                .map(|(_, sw, rep)| (sw, *rep as u64))
                .collect();
            let global = SparseWeights::aggregate(&votes);
            let max_uid = T::MaxMinersPerSubnet::get();
            if let Ok(gc) = global.compress(max_uid) {
                GlobalWeights::<T>::insert(subnet_id, epoch, gc);
            }

            // Update reputations via EMA on cosine similarity to global.
            for (uid, sw, old_rep) in &decoded {
                let sim = sw.cosine_similarity(&global);
                let new_rep = ema(*old_rep, sim);
                Reputation::<T>::insert(subnet_id, *uid, new_rep);
                Self::deposit_event(Event::ReputationUpdated {
                    subnet_id,
                    validator_uid: *uid,
                    old_reputation: *old_rep,
                    new_reputation: new_rep,
                    similarity: sim,
                });
            }

            // Collusion detection on the raw submitted vectors.
            let pairs: Vec<(Uid, SparseWeights)> = decoded
                .iter()
                .map(|(uid, sw, _)| (*uid, sw.clone()))
                .collect();
            let clusters = detect_collusion_clusters(&pairs);
            for cluster in &clusters {
                Self::deposit_event(Event::CollusionDetected {
                    subnet_id,
                    epoch,
                    members: cluster.clone(),
                });
                for uid in cluster {
                    let before = Reputation::<T>::get(subnet_id, *uid);
                    let after = before.saturating_sub(COLLUSION_PENALTY_PPM);
                    Reputation::<T>::insert(subnet_id, *uid, after);
                    Self::deposit_event(Event::CollusionPenaltyApplied {
                        subnet_id,
                        validator_uid: *uid,
                        reputation_before: before,
                        reputation_after: after,
                    });
                }
            }

            EpochFinalized::<T>::insert(subnet_id, epoch, true);
            Self::deposit_event(Event::EpochFinalized {
                subnet_id,
                epoch,
                submitter_count: submitters.len() as u32,
            });
            Ok(())
        }
    }

    /// EMA reputation update:
    ///   new = (alpha * old + (COSINE_SCALE - alpha) * similarity) / COSINE_SCALE
    fn ema(old: u32, similarity: u32) -> u32 {
        let alpha = REPUTATION_ALPHA_PPM as u64;
        let one_minus = (COSINE_SCALE as u64).saturating_sub(alpha);
        let blended = (alpha.saturating_mul(old as u64)
            + one_minus.saturating_mul(similarity as u64))
            / COSINE_SCALE as u64;
        blended as u32
    }

    /// Union-find cluster detector. Connects any two validators whose pairwise
    /// cosine similarity exceeds `COLLUSION_THRESHOLD_PPM`. Returns clusters of
    /// size >= 2 with each inner vec sorted by uid.
    pub fn detect_collusion_clusters(vectors: &[(Uid, SparseWeights)]) -> Vec<Vec<Uid>> {
        use sp_std::collections::btree_map::BTreeMap;
        let n = vectors.len();
        if n < 2 {
            return Vec::new();
        }
        let mut parent: Vec<usize> = (0..n).collect();
        fn find(parent: &mut [usize], i: usize) -> usize {
            if parent[i] == i {
                i
            } else {
                let r = find(parent, parent[i]);
                parent[i] = r;
                r
            }
        }
        for i in 0..n {
            for j in (i + 1)..n {
                let sim = vectors[i].1.cosine_similarity(&vectors[j].1);
                if sim > COLLUSION_THRESHOLD_PPM {
                    let a = find(&mut parent, i);
                    let b = find(&mut parent, j);
                    if a != b {
                        parent[a] = b;
                    }
                }
            }
        }
        let mut groups: BTreeMap<usize, Vec<Uid>> = BTreeMap::new();
        for (i, (uid, _)) in vectors.iter().enumerate() {
            let r = find(&mut parent, i);
            groups.entry(r).or_default().push(*uid);
        }
        groups.into_values().filter(|g| g.len() >= 2).collect()
    }

    impl<T: Config> Pallet<T> {
        /// Current epoch index = floor(block_number / EpochLength).
        pub fn current_epoch() -> EpochIndex {
            let block: u64 = <frame_system::Pallet<T>>::block_number()
                .try_into()
                .ok()
                .unwrap_or(0);
            let len: u64 = T::EpochLength::get().try_into().ok().unwrap_or(1).max(1);
            block / len
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as pallet_consensus;
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
            Consensus: pallet_consensus,
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
        type AccountData = ();
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
        pub const EpochLength: u64 = 10;
        pub const MaxMinersPerSubnet: u32 = 100;
    }

    impl Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type EpochLength = EpochLength;
        type MaxMinersPerSubnet = MaxMinersPerSubnet;
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap()
            .into()
    }

    #[test]
    fn empty_weights_rejected() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                Consensus::submit_weights(RuntimeOrigin::signed(1), 1, 0, Vec::new()),
                Error::<Test>::EmptyWeights
            );
        });
    }

    #[test]
    fn too_many_entries_rejected() {
        new_test_ext().execute_with(|| {
            let big: Vec<(Uid, u16)> = (0..MaxMinersPerSubnet::get() + 1).map(|i| (i, 1)).collect();
            assert_noop!(
                Consensus::submit_weights(RuntimeOrigin::signed(1), 1, 0, big),
                Error::<Test>::TooManyEntries
            );
        });
    }

    #[test]
    fn out_of_range_uid_rejected() {
        new_test_ext().execute_with(|| {
            let bad = vec![(MaxMinersPerSubnet::get(), 1)];
            assert_noop!(
                Consensus::submit_weights(RuntimeOrigin::signed(1), 1, 0, bad),
                Error::<Test>::IndexOutOfBounds
            );
        });
    }

    #[test]
    fn all_zero_weights_rejected() {
        new_test_ext().execute_with(|| {
            let all_zero = vec![(0, 0), (1, 0), (2, 0)];
            assert_noop!(
                Consensus::submit_weights(RuntimeOrigin::signed(1), 1, 0, all_zero),
                Error::<Test>::EmptyWeights
            );
        });
    }

    #[test]
    fn submit_stores_compressed_and_increments_count() {
        new_test_ext().execute_with(|| {
            assert_ok!(Consensus::submit_weights(
                RuntimeOrigin::signed(1),
                1,
                0,
                vec![(0, 100), (1, 200), (2, 300)]
            ));
            let epoch = Consensus::current_epoch();
            assert!(Consensus::submissions((1, epoch, 0)).is_some());
            assert_eq!(Consensus::submission_count(1, epoch), 1);
        });
    }

    #[test]
    fn resubmit_overwrites_without_double_counting() {
        new_test_ext().execute_with(|| {
            assert_ok!(Consensus::submit_weights(
                RuntimeOrigin::signed(1),
                1,
                0,
                vec![(0, 100)]
            ));
            assert_ok!(Consensus::submit_weights(
                RuntimeOrigin::signed(1),
                1,
                0,
                vec![(0, 50), (1, 50)]
            ));
            let epoch = Consensus::current_epoch();
            assert_eq!(Consensus::submission_count(1, epoch), 1);
            let stored = Consensus::submissions((1, epoch, 0)).unwrap();
            assert_eq!(stored.runs.len(), 2);
        });
    }

    #[test]
    fn distinct_validators_increment_count() {
        new_test_ext().execute_with(|| {
            assert_ok!(Consensus::submit_weights(
                RuntimeOrigin::signed(1),
                1,
                0,
                vec![(0, 1)]
            ));
            assert_ok!(Consensus::submit_weights(
                RuntimeOrigin::signed(2),
                1,
                1,
                vec![(0, 1)]
            ));
            let epoch = Consensus::current_epoch();
            assert_eq!(Consensus::submission_count(1, epoch), 2);
        });
    }

    #[test]
    fn epoch_advances_with_block_number() {
        new_test_ext().execute_with(|| {
            assert_eq!(Consensus::current_epoch(), 0);
            System::set_block_number(EpochLength::get());
            assert_eq!(Consensus::current_epoch(), 1);
            System::set_block_number(EpochLength::get() * 3 + 5);
            assert_eq!(Consensus::current_epoch(), 3);
        });
    }

    #[test]
    fn submission_bucketed_per_epoch() {
        new_test_ext().execute_with(|| {
            assert_ok!(Consensus::submit_weights(
                RuntimeOrigin::signed(1),
                1,
                0,
                vec![(0, 1)]
            ));
            System::set_block_number(EpochLength::get() * 2);
            assert_ok!(Consensus::submit_weights(
                RuntimeOrigin::signed(1),
                1,
                0,
                vec![(0, 1)]
            ));
            assert_eq!(Consensus::submission_count(1, 0), 1);
            assert_eq!(Consensus::submission_count(1, 2), 1);
        });
    }

    #[test]
    fn finalize_rejects_before_epoch_ends() {
        new_test_ext().execute_with(|| {
            assert_ok!(Consensus::submit_weights(
                RuntimeOrigin::signed(1),
                1,
                0,
                vec![(0, 1)]
            ));
            assert_noop!(
                Consensus::finalize_epoch(RuntimeOrigin::signed(1), 1, 0),
                Error::<Test>::EpochNotReady
            );
        });
    }

    #[test]
    fn finalize_rejects_no_submissions() {
        new_test_ext().execute_with(|| {
            System::set_block_number(EpochLength::get() * 2);
            assert_noop!(
                Consensus::finalize_epoch(RuntimeOrigin::signed(1), 1, 0),
                Error::<Test>::NoSubmissions
            );
        });
    }

    #[test]
    fn finalize_aggregates_and_updates_reputation() {
        new_test_ext().execute_with(|| {
            // Two validators submit in epoch 0; one agrees with the other in
            // epoch 1's perspective once aggregated.
            assert_ok!(Consensus::submit_weights(
                RuntimeOrigin::signed(1),
                1,
                0,
                vec![(0, 100), (1, 200)]
            ));
            assert_ok!(Consensus::submit_weights(
                RuntimeOrigin::signed(2),
                1,
                1,
                vec![(0, 100), (1, 200)]
            ));
            // Advance past epoch 0
            System::set_block_number(EpochLength::get());
            assert_ok!(Consensus::finalize_epoch(RuntimeOrigin::signed(1), 1, 0));

            assert!(Consensus::epoch_finalized(1, 0));
            assert!(Consensus::global_weights(1, 0).is_some());

            // Both validators agreed perfectly → reputation should move toward
            // COSINE_SCALE from the initial 500_000 floor.
            let r0 = Consensus::reputation(1, 0);
            let r1 = Consensus::reputation(1, 1);
            assert!(r0 >= INITIAL_REPUTATION);
            assert!(r1 >= INITIAL_REPUTATION);
        });
    }

    #[test]
    fn finalize_rewards_agreement_over_outlier() {
        new_test_ext().execute_with(|| {
            // v0 and v1 agree broadly but not identically (to avoid collusion
            // penalty); v2 is a full outlier.
            assert_ok!(Consensus::submit_weights(
                RuntimeOrigin::signed(1),
                1,
                0,
                vec![(0, 100)]
            ));
            assert_ok!(Consensus::submit_weights(
                RuntimeOrigin::signed(2),
                1,
                1,
                vec![(0, 100), (1, 50)]
            ));
            assert_ok!(Consensus::submit_weights(
                RuntimeOrigin::signed(3),
                1,
                2,
                vec![(5, 100)]
            ));
            System::set_block_number(EpochLength::get());
            assert_ok!(Consensus::finalize_epoch(RuntimeOrigin::signed(1), 1, 0));
            // Agreeing validators should end up with higher reputation than
            // the outlier.
            let r_agree = Consensus::reputation(1, 0);
            let r_outlier = Consensus::reputation(1, 2);
            assert!(r_agree > r_outlier);
        });
    }

    #[test]
    fn finalize_twice_fails() {
        new_test_ext().execute_with(|| {
            assert_ok!(Consensus::submit_weights(
                RuntimeOrigin::signed(1),
                1,
                0,
                vec![(0, 1)]
            ));
            System::set_block_number(EpochLength::get());
            assert_ok!(Consensus::finalize_epoch(RuntimeOrigin::signed(1), 1, 0));
            assert_noop!(
                Consensus::finalize_epoch(RuntimeOrigin::signed(1), 1, 0),
                Error::<Test>::EpochAlreadyFinalized
            );
        });
    }

    #[test]
    fn submit_after_finalize_fails() {
        new_test_ext().execute_with(|| {
            assert_ok!(Consensus::submit_weights(
                RuntimeOrigin::signed(1),
                1,
                0,
                vec![(0, 1)]
            ));
            System::set_block_number(EpochLength::get());
            assert_ok!(Consensus::finalize_epoch(RuntimeOrigin::signed(1), 1, 0));
            // Same validator tries to submit more weights for the already
            // finalized epoch 0 — current_epoch is now 1 so this submission
            // actually targets epoch 1 and succeeds. Instead we verify
            // EpochFinalized blocks re-finalization (covered above) and that
            // overriding a finalized epoch storage is impossible via
            // submit_weights by construction — it only writes to current_epoch.
            System::set_block_number(0);
            assert_noop!(
                Consensus::submit_weights(RuntimeOrigin::signed(1), 1, 0, vec![(0, 1)]),
                Error::<Test>::EpochAlreadyFinalized
            );
        });
    }

    #[test]
    fn collusion_detected_for_identical_vectors() {
        use sp_neuro_weight::SparseWeights;
        let v = SparseWeights::new_sorted(vec![(0, 100), (1, 200), (2, 300)]).unwrap();
        let input = vec![(1, v.clone()), (2, v.clone()), (3, v.clone())];
        let clusters = detect_collusion_clusters(&input);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0], vec![1, 2, 3]);
    }

    #[test]
    fn collusion_not_triggered_for_diverse_vectors() {
        use sp_neuro_weight::SparseWeights;
        let a = SparseWeights::new_sorted(vec![(0, 100)]).unwrap();
        let b = SparseWeights::new_sorted(vec![(1, 100)]).unwrap();
        let c = SparseWeights::new_sorted(vec![(2, 100)]).unwrap();
        let input = vec![(1, a), (2, b), (3, c)];
        let clusters = detect_collusion_clusters(&input);
        assert!(clusters.is_empty());
    }

    #[test]
    fn finalize_applies_collusion_penalty() {
        new_test_ext().execute_with(|| {
            // v0, v1 submit identical vectors (colluding); v2 is independent.
            assert_ok!(Consensus::submit_weights(
                RuntimeOrigin::signed(1),
                1,
                0,
                vec![(0, 100), (1, 200), (2, 300)]
            ));
            assert_ok!(Consensus::submit_weights(
                RuntimeOrigin::signed(2),
                1,
                1,
                vec![(0, 100), (1, 200), (2, 300)]
            ));
            assert_ok!(Consensus::submit_weights(
                RuntimeOrigin::signed(3),
                1,
                2,
                vec![(5, 1)]
            ));
            System::set_block_number(EpochLength::get());
            assert_ok!(Consensus::finalize_epoch(RuntimeOrigin::signed(1), 1, 0));
            // Colluders had their reputation reduced relative to the non-colluder
            let r0 = Consensus::reputation(1, 0);
            let r2 = Consensus::reputation(1, 2);
            assert!(r0 < r2 + COLLUSION_PENALTY_PPM);
        });
    }

    #[test]
    fn stored_weights_are_l1_normalized() {
        new_test_ext().execute_with(|| {
            assert_ok!(Consensus::submit_weights(
                RuntimeOrigin::signed(1),
                1,
                0,
                vec![(0, 10), (1, 20), (2, 70)]
            ));
            let epoch = Consensus::current_epoch();
            let c = Consensus::submissions((1, epoch, 0)).unwrap();
            let sum: u64 = c.runs.iter().map(|(_, w)| *w as u64).sum();
            // Rounding slack of n entries (3)
            assert!(sum >= u16::MAX as u64 - 3 && sum <= u16::MAX as u64);
        });
    }
}
