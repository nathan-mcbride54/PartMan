//! Pure loop-acceptance protocol and evidence-publication gate.

use crate::{FixtureRole, Refusal, RunReport};

pub(super) const REQUIRED_BLOCK_SIZE: u32 = 512;
pub(super) const FLAG_READ_ONLY: u32 = 1;
pub(super) const FLAG_AUTOCLEAR: u32 = 4;
pub(super) const FLAG_PARTSCAN: u32 = 8;
pub(super) const REQUIRED_FLAGS: u32 = FLAG_READ_ONLY | FLAG_AUTOCLEAR | FLAG_PARTSCAN;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ConfigureRequest {
    pub(super) offset: u64,
    pub(super) size_limit: u64,
    pub(super) block_size: u32,
    pub(super) flags: u32,
}

impl ConfigureRequest {
    pub(super) const READ_ONLY_ACCEPTANCE: Self = Self {
        offset: 0,
        size_limit: 0,
        block_size: REQUIRED_BLOCK_SIZE,
        flags: REQUIRED_FLAGS,
    };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ConfigureError {
    Busy,
    Refused(Refusal),
}

pub(super) trait Controller {
    type Attachment;

    fn expected_digest(&self, fixture: FixtureRole) -> [u8; 32];

    fn digest(&mut self, fixture: FixtureRole) -> Result<[u8; 32], Refusal>;

    fn configure(
        &mut self,
        fixture: FixtureRole,
        request: ConfigureRequest,
    ) -> Result<Self::Attachment, ConfigureError>;

    fn verify(
        &mut self,
        attachment: &Self::Attachment,
        expected: FixtureRole,
    ) -> Result<(), Refusal>;

    fn probe(&mut self, attachment: &Self::Attachment) -> Result<usize, Refusal>;

    fn rebind(
        &mut self,
        attachment: &Self::Attachment,
        replacement: FixtureRole,
    ) -> Result<(), Refusal>;

    fn detach(&mut self, attachment: Self::Attachment) -> Result<(), Refusal>;
}

#[derive(Debug, Default)]
struct EvidenceGate {
    pending_bytes: Option<usize>,
    final_verified: bool,
    detached: bool,
    discarded: bool,
}

impl EvidenceGate {
    fn observe(&mut self, bytes: usize) -> Result<(), Refusal> {
        if self.pending_bytes.is_some() || self.final_verified || self.detached {
            return Err(Refusal::ProtocolOrder);
        }
        self.pending_bytes = Some(bytes);
        Ok(())
    }

    fn verify(&mut self) -> Result<(), Refusal> {
        if self.pending_bytes.is_none() || self.detached {
            return Err(Refusal::ProtocolOrder);
        }
        self.final_verified = true;
        Ok(())
    }

    fn discard(&mut self) -> Result<(), Refusal> {
        if self.pending_bytes.take().is_none() || self.detached {
            return Err(Refusal::ProtocolOrder);
        }
        self.final_verified = false;
        self.discarded = true;
        Ok(())
    }

    fn detached(&mut self) -> Result<(), Refusal> {
        if self.detached {
            return Err(Refusal::ProtocolOrder);
        }
        self.detached = true;
        Ok(())
    }

    fn publish(self) -> Result<usize, Refusal> {
        if !self.final_verified || !self.detached || self.discarded {
            return Err(Refusal::ProtocolOrder);
        }
        self.pending_bytes.ok_or(Refusal::ProtocolOrder)
    }

    fn prove_discarded(self) -> Result<(), Refusal> {
        if self.discarded && self.pending_bytes.is_none() && !self.final_verified && self.detached {
            Ok(())
        } else {
            Err(Refusal::ProtocolOrder)
        }
    }
}

pub(super) fn execute<C: Controller>(controller: &mut C) -> Result<RunReport, Refusal> {
    let basic_before = controller.digest(FixtureRole::Basic)?;
    if basic_before != controller.expected_digest(FixtureRole::Basic) {
        return Err(Refusal::InitialFixtureHashMismatch {
            fixture: FixtureRole::Basic,
        });
    }
    let conflicting_before = controller.digest(FixtureRole::Conflicting)?;
    if conflicting_before != controller.expected_digest(FixtureRole::Conflicting) {
        return Err(Refusal::InitialFixtureHashMismatch {
            fixture: FixtureRole::Conflicting,
        });
    }
    let clean = configure_once(controller)?;
    let mut clean_gate = EvidenceGate::default();
    let clean_result = (|| {
        controller.verify(&clean, FixtureRole::Basic)?;
        clean_gate.observe(controller.probe(&clean)?)?;
        controller.verify(&clean, FixtureRole::Basic)?;
        clean_gate.verify()
    })();
    let clean_detach = controller.detach(clean);
    clean_detach?;
    clean_gate.detached()?;
    clean_result?;

    let adversarial = configure_once(controller)?;
    let mut adversarial_gate = EvidenceGate::default();
    let adversarial_result = (|| {
        controller.verify(&adversarial, FixtureRole::Basic)?;
        adversarial_gate.observe(controller.probe(&adversarial)?)?;
        controller.rebind(&adversarial, FixtureRole::Conflicting)?;
        // A difference from Basic is not enough: an arbitrary third backing
        // would also differ. First bind the status positively to the exact held
        // Conflicting descriptor, then require the complementary Basic check
        // to fail before discarding the pending observation.
        controller.verify(&adversarial, FixtureRole::Conflicting)?;
        match controller.verify(&adversarial, FixtureRole::Basic) {
            Err(Refusal::BackingIdentityMismatch) => adversarial_gate.discard(),
            Ok(()) => Err(Refusal::AdversarialRebindNotDetected),
            Err(error) => Err(error),
        }
    })();
    let adversarial_detach = controller.detach(adversarial);
    adversarial_detach?;
    adversarial_gate.detached()?;
    adversarial_result?;
    adversarial_gate.prove_discarded()?;

    if controller.digest(FixtureRole::Basic)? != basic_before {
        return Err(Refusal::FixtureHashChanged {
            fixture: FixtureRole::Basic,
        });
    }
    if controller.digest(FixtureRole::Conflicting)? != conflicting_before {
        return Err(Refusal::FixtureHashChanged {
            fixture: FixtureRole::Conflicting,
        });
    }

    // Aggregate publication is intentionally last. The clean bytes stay
    // sealed through the adversarial leg, both confirmed detaches, and both
    // post-run hashes; no earlier success object exists for a caller to use.
    let clean_observation_bytes = clean_gate.publish()?;

    Ok(RunReport {
        configured_legs: 2,
        clean_observation_bytes,
        detachments_confirmed: 2,
    })
}

fn configure_once<C: Controller>(controller: &mut C) -> Result<C::Attachment, Refusal> {
    match controller.configure(FixtureRole::Basic, ConfigureRequest::READ_ONLY_ACCEPTANCE) {
        Ok(attachment) => Ok(attachment),
        Err(ConfigureError::Busy) => Err(Refusal::LoopIsolationConflict),
        Err(ConfigureError::Refused(error)) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[derive(Debug)]
    struct FakeAttachment(u8);

    #[derive(Debug, PartialEq, Eq)]
    enum Event {
        Digest(FixtureRole),
        Configure {
            fixture: FixtureRole,
            request: ConfigureRequest,
            attachment: Option<u8>,
        },
        Verify {
            attachment: u8,
            expected: FixtureRole,
        },
        Probe {
            attachment: u8,
        },
        Rebind {
            attachment: u8,
            replacement: FixtureRole,
        },
        Detach {
            attachment: u8,
        },
    }

    struct FakeController {
        configure: VecDeque<Result<FakeAttachment, ConfigureError>>,
        verify: VecDeque<Result<(), Refusal>>,
        probes: VecDeque<Result<usize, Refusal>>,
        rebind: Result<(), Refusal>,
        detach: VecDeque<Result<(), Refusal>>,
        digests: VecDeque<Result<[u8; 32], Refusal>>,
        digest_calls: usize,
        requests: Vec<ConfigureRequest>,
        detached_ids: Vec<u8>,
        verified_roles: Vec<FixtureRole>,
        events: Vec<Event>,
        expected_basic: [u8; 32],
        expected_conflicting: [u8; 32],
    }

    impl FakeController {
        fn success() -> Self {
            Self {
                configure: VecDeque::from([Ok(FakeAttachment(1)), Ok(FakeAttachment(2))]),
                verify: VecDeque::from([
                    Ok(()),
                    Ok(()),
                    Ok(()),
                    Ok(()),
                    Err(Refusal::BackingIdentityMismatch),
                ]),
                probes: VecDeque::from([Ok(4096), Ok(4096)]),
                rebind: Ok(()),
                detach: VecDeque::from([Ok(()), Ok(())]),
                digests: VecDeque::from([Ok([1; 32]), Ok([2; 32]), Ok([1; 32]), Ok([2; 32])]),
                digest_calls: 0,
                requests: Vec::new(),
                detached_ids: Vec::new(),
                verified_roles: Vec::new(),
                events: Vec::new(),
                expected_basic: [1; 32],
                expected_conflicting: [2; 32],
            }
        }
    }

    impl Controller for FakeController {
        type Attachment = FakeAttachment;

        fn expected_digest(&self, fixture: FixtureRole) -> [u8; 32] {
            match fixture {
                FixtureRole::Basic => self.expected_basic,
                FixtureRole::Conflicting => self.expected_conflicting,
            }
        }

        fn digest(&mut self, fixture: FixtureRole) -> Result<[u8; 32], Refusal> {
            self.events.push(Event::Digest(fixture));
            self.digest_calls += 1;
            self.digests
                .pop_front()
                .expect("fake digest script is complete")
        }

        fn configure(
            &mut self,
            fixture: FixtureRole,
            request: ConfigureRequest,
        ) -> Result<Self::Attachment, ConfigureError> {
            self.requests.push(request);
            let result = self
                .configure
                .pop_front()
                .expect("fake configure script is complete");
            self.events.push(Event::Configure {
                fixture,
                request,
                attachment: result.as_ref().ok().map(|attachment| attachment.0),
            });
            result
        }

        fn verify(
            &mut self,
            attachment: &Self::Attachment,
            expected: FixtureRole,
        ) -> Result<(), Refusal> {
            self.events.push(Event::Verify {
                attachment: attachment.0,
                expected,
            });
            self.verified_roles.push(expected);
            self.verify
                .pop_front()
                .expect("fake verify script is complete")
        }

        fn probe(&mut self, attachment: &Self::Attachment) -> Result<usize, Refusal> {
            self.events.push(Event::Probe {
                attachment: attachment.0,
            });
            self.probes
                .pop_front()
                .expect("fake probe script is complete")
        }

        fn rebind(
            &mut self,
            attachment: &Self::Attachment,
            replacement: FixtureRole,
        ) -> Result<(), Refusal> {
            self.events.push(Event::Rebind {
                attachment: attachment.0,
                replacement,
            });
            self.rebind.clone()
        }

        fn detach(&mut self, attachment: Self::Attachment) -> Result<(), Refusal> {
            self.events.push(Event::Detach {
                attachment: attachment.0,
            });
            self.detached_ids.push(attachment.0);
            self.detach
                .pop_front()
                .expect("fake detach script is complete")
        }
    }

    // Requirements: SAFE-001, SAFE-007, Section 11.3
    //   The request is read-only and fixes every loop configuration field used by Tier 2.
    // Evidence: request_is_exactly_read_only_autoclear_partscan_at_512_bytes
    #[test]
    fn request_is_exactly_read_only_autoclear_partscan_at_512_bytes() {
        let request = ConfigureRequest::READ_ONLY_ACCEPTANCE;
        assert_eq!(
            request.flags,
            FLAG_READ_ONLY | FLAG_AUTOCLEAR | FLAG_PARTSCAN
        );
        assert_eq!(request.block_size, 512);
        assert_eq!(request.offset, 0);
        assert_eq!(request.size_limit, 0);
    }

    // Requirements: SAFE-001, SAFE-005, SAFE-007
    //   Stable but already-mutated starting bytes cannot satisfy a before/after equality check.
    // Evidence: wrong_starting_hash_refuses_before_any_loop_configuration
    #[test]
    fn wrong_starting_hash_refuses_before_any_loop_configuration() {
        for (fixture, digests, digest_calls) in [
            (
                FixtureRole::Basic,
                VecDeque::from([Ok([9; 32]), Ok([2; 32]), Ok([9; 32]), Ok([2; 32])]),
                1,
            ),
            (
                FixtureRole::Conflicting,
                VecDeque::from([Ok([1; 32]), Ok([9; 32]), Ok([1; 32]), Ok([9; 32])]),
                2,
            ),
        ] {
            let mut fake = FakeController::success();
            // The queued post-run hashes equal the wrong starting hashes. The
            // compiled expectation must refuse before that equality can pass.
            fake.digests = digests;
            assert_eq!(
                execute(&mut fake),
                Err(Refusal::InitialFixtureHashMismatch { fixture })
            );
            assert_eq!(fake.digest_calls, digest_calls);
            assert!(fake.requests.is_empty());
            assert!(fake.detached_ids.is_empty());
        }
    }

    // Requirements: SAFE-005, SAFE-007
    //   No observation publishes before clean final verification, both detach confirmations,
    //   adversarial discard, and both unchanged-fixture hashes.
    // Evidence: clean_and_adversarial_legs_publish_only_after_verify_and_detach
    #[test]
    fn clean_and_adversarial_legs_publish_only_after_verify_and_detach() {
        let mut fake = FakeController::success();
        let report = execute(&mut fake).expect("complete scripted proof");
        assert_eq!(report.configured_legs(), 2);
        assert_eq!(report.clean_observation_bytes(), 4096);
        assert!(report.required_configuration_verified());
        assert!(report.adversarial_rebind_detected());
        assert!(report.adversarial_observation_discarded());
        assert_eq!(report.detachments_confirmed(), 2);
        assert!(report.partition_teardown_confirmed());
        assert!(report.initial_fixture_hashes_matched_catalogue());
        assert!(report.fixture_hashes_unchanged());
        assert_eq!(fake.detached_ids, [1, 2]);
        assert_eq!(
            fake.verified_roles,
            [
                FixtureRole::Basic,
                FixtureRole::Basic,
                FixtureRole::Basic,
                FixtureRole::Conflicting,
                FixtureRole::Basic,
            ]
        );
        assert_eq!(
            fake.events,
            vec![
                Event::Digest(FixtureRole::Basic),
                Event::Digest(FixtureRole::Conflicting),
                Event::Configure {
                    fixture: FixtureRole::Basic,
                    request: ConfigureRequest::READ_ONLY_ACCEPTANCE,
                    attachment: Some(1),
                },
                Event::Verify {
                    attachment: 1,
                    expected: FixtureRole::Basic,
                },
                Event::Probe { attachment: 1 },
                Event::Verify {
                    attachment: 1,
                    expected: FixtureRole::Basic,
                },
                Event::Detach { attachment: 1 },
                Event::Configure {
                    fixture: FixtureRole::Basic,
                    request: ConfigureRequest::READ_ONLY_ACCEPTANCE,
                    attachment: Some(2),
                },
                Event::Verify {
                    attachment: 2,
                    expected: FixtureRole::Basic,
                },
                Event::Probe { attachment: 2 },
                Event::Rebind {
                    attachment: 2,
                    replacement: FixtureRole::Conflicting,
                },
                Event::Verify {
                    attachment: 2,
                    expected: FixtureRole::Conflicting,
                },
                Event::Verify {
                    attachment: 2,
                    expected: FixtureRole::Basic,
                },
                Event::Detach { attachment: 2 },
                Event::Digest(FixtureRole::Basic),
                Event::Digest(FixtureRole::Conflicting),
            ]
        );
        assert!(
            fake.requests
                .iter()
                .all(|request| *request == ConfigureRequest::READ_ONLY_ACCEPTANCE)
        );
    }

    // Requirements: SAFE-005, SAFE-007
    //   A busy configure means isolated loop state was not established and refuses
    //   without retry.
    // Evidence: first_configure_ebusy_refuses_without_retry_or_attachment
    #[test]
    fn first_configure_ebusy_refuses_without_retry_or_attachment() {
        let mut fake = FakeController::success();
        fake.configure.push_front(Err(ConfigureError::Busy));
        assert_eq!(execute(&mut fake), Err(Refusal::LoopIsolationConflict));
        assert_eq!(fake.requests.len(), 1);
        assert!(fake.detached_ids.is_empty());
    }

    #[test]
    fn second_configure_ebusy_refuses_after_confirmed_clean_detach() {
        let mut fake = FakeController::success();
        fake.configure = VecDeque::from([
            Ok(FakeAttachment(1)),
            Err(ConfigureError::Busy),
            Ok(FakeAttachment(2)),
        ]);
        assert_eq!(execute(&mut fake), Err(Refusal::LoopIsolationConflict));
        assert_eq!(fake.requests.len(), 2);
        assert_eq!(fake.detached_ids, [1]);
    }

    #[test]
    fn a_non_busy_configure_refusal_is_not_retried() {
        let mut fake = FakeController::success();
        fake.configure = VecDeque::from([Err(ConfigureError::Refused(Refusal::KernelOperation {
            operation: "loop-configure",
            errno: Some(1),
        }))]);
        assert_eq!(
            execute(&mut fake),
            Err(Refusal::KernelOperation {
                operation: "loop-configure",
                errno: Some(1),
            })
        );
        assert!(fake.detached_ids.is_empty());
    }

    // Requirements: SAFE-005, SAFE-007
    //   Every status/configuration mismatch refuses and the attachment is still cleaned up.
    // Evidence: every_clean_mismatch_refuses_and_still_detaches
    #[test]
    fn every_clean_mismatch_refuses_and_still_detaches() {
        let mismatches = [
            Refusal::BackingIdentityMismatch,
            Refusal::LoopNodeIdentityMismatch,
            Refusal::LoopFlagsMismatch,
            Refusal::LoopGeometryMismatch,
            Refusal::BlockSizeMismatch,
            Refusal::LoopNumberMismatch,
        ];
        for mismatch in mismatches {
            let mut fake = FakeController::success();
            fake.verify = VecDeque::from([Err(mismatch.clone())]);
            assert_eq!(execute(&mut fake), Err(mismatch));
            assert_eq!(fake.detached_ids, [1]);
        }
    }

    #[test]
    fn a_probe_failure_refuses_and_still_detaches() {
        let mut fake = FakeController::success();
        fake.probes = VecDeque::from([Err(Refusal::ProbeFailed { errno: Some(5) })]);
        assert_eq!(
            execute(&mut fake),
            Err(Refusal::ProbeFailed { errno: Some(5) })
        );
        assert_eq!(fake.detached_ids, [1]);
    }

    // Requirements: SAFE-005, SAFE-007
    //   A distinct post-probe identity failure withholds the pending clean observation.
    // Evidence: a_post_probe_verification_failure_withholds_the_pending_clean_observation
    #[test]
    fn a_post_probe_verification_failure_withholds_the_pending_clean_observation() {
        let mut fake = FakeController::success();
        fake.verify = VecDeque::from([Ok(()), Err(Refusal::LoopNodeIdentityMismatch)]);
        assert_eq!(execute(&mut fake), Err(Refusal::LoopNodeIdentityMismatch));
        assert_eq!(
            fake.events,
            vec![
                Event::Digest(FixtureRole::Basic),
                Event::Digest(FixtureRole::Conflicting),
                Event::Configure {
                    fixture: FixtureRole::Basic,
                    request: ConfigureRequest::READ_ONLY_ACCEPTANCE,
                    attachment: Some(1),
                },
                Event::Verify {
                    attachment: 1,
                    expected: FixtureRole::Basic,
                },
                Event::Probe { attachment: 1 },
                Event::Verify {
                    attachment: 1,
                    expected: FixtureRole::Basic,
                },
                Event::Detach { attachment: 1 },
            ]
        );
    }

    // Requirements: SAFE-005, SAFE-007
    //   Cleanup failure suppresses otherwise valid pending evidence.
    // Evidence: cleanup_failure_overrides_an_otherwise_publishable_observation
    #[test]
    fn cleanup_failure_overrides_an_otherwise_publishable_observation() {
        let mut fake = FakeController::success();
        fake.detach = VecDeque::from([Err(Refusal::DetachNotConfirmed)]);
        assert_eq!(execute(&mut fake), Err(Refusal::DetachNotConfirmed));
        assert_eq!(fake.digest_calls, 2, "no post-run hash may imply success");
    }

    // Requirements: SAFE-005, SAFE-007
    //   A successful adversarial rebind must be detected by backing identity read-back.
    // Evidence: successful_rebind_that_verification_misses_refuses
    #[test]
    fn successful_rebind_that_verification_misses_refuses() {
        let mut fake = FakeController::success();
        fake.verify = VecDeque::from([Ok(()), Ok(()), Ok(()), Ok(()), Ok(())]);
        assert_eq!(
            execute(&mut fake),
            Err(Refusal::AdversarialRebindNotDetected)
        );
        assert_eq!(fake.detached_ids, [1, 2]);
    }

    #[test]
    fn rebind_failure_refuses_and_still_detaches() {
        let mut fake = FakeController::success();
        fake.rebind = Err(Refusal::AdversarialRebindFailed { errno: Some(22) });
        assert_eq!(
            execute(&mut fake),
            Err(Refusal::AdversarialRebindFailed { errno: Some(22) })
        );
        assert_eq!(fake.detached_ids, [1, 2]);
    }

    #[test]
    fn an_arbitrary_non_basic_replacement_cannot_satisfy_the_adversarial_proof() {
        let mut fake = FakeController::success();
        fake.verify = VecDeque::from([
            Ok(()),
            Ok(()),
            Ok(()),
            Err(Refusal::BackingIdentityMismatch),
        ]);
        assert_eq!(execute(&mut fake), Err(Refusal::BackingIdentityMismatch));
        assert_eq!(fake.detached_ids, [1, 2]);
        assert_eq!(
            fake.verified_roles,
            [
                FixtureRole::Basic,
                FixtureRole::Basic,
                FixtureRole::Basic,
                FixtureRole::Conflicting,
            ]
        );
    }

    #[test]
    fn adversarial_precheck_failure_still_detaches_the_second_attachment() {
        let mut fake = FakeController::success();
        fake.verify = VecDeque::from([Ok(()), Ok(()), Err(Refusal::LoopNodeIdentityMismatch)]);
        assert_eq!(execute(&mut fake), Err(Refusal::LoopNodeIdentityMismatch));
        assert_eq!(fake.detached_ids, [1, 2]);
    }

    // Requirements: SAFE-005, SAFE-007
    //   A second-leg cleanup failure overrides the expected adversarial mismatch.
    // Evidence: adversarial_cleanup_failure_overrides_the_expected_rebind_mismatch
    #[test]
    fn adversarial_cleanup_failure_overrides_the_expected_rebind_mismatch() {
        let mut fake = FakeController::success();
        fake.detach = VecDeque::from([Ok(()), Err(Refusal::DetachNotConfirmed)]);
        assert_eq!(execute(&mut fake), Err(Refusal::DetachNotConfirmed));
        assert_eq!(fake.detached_ids, [1, 2]);
    }

    // Requirements: SAFE-001, SAFE-005, SAFE-007
    //   Both authorized fixture hashes gate aggregate evidence after both detachments.
    // Evidence: either_fixture_hash_change_refuses_after_both_detaches
    #[test]
    fn either_fixture_hash_change_refuses_after_both_detaches() {
        let cases = [
            (
                FixtureRole::Basic,
                VecDeque::from([Ok([1; 32]), Ok([2; 32]), Ok([3; 32])]),
            ),
            (
                FixtureRole::Conflicting,
                VecDeque::from([Ok([1; 32]), Ok([2; 32]), Ok([1; 32]), Ok([4; 32])]),
            ),
        ];
        for (role, digests) in cases {
            let mut fake = FakeController::success();
            fake.digests = digests;
            assert_eq!(
                execute(&mut fake),
                Err(Refusal::FixtureHashChanged { fixture: role })
            );
            assert_eq!(fake.detached_ids, [1, 2]);
        }
    }

    // Requirements: SAFE-005, SAFE-007
    //   The clean observation stays sealed until the last post-run hash succeeds.
    // Evidence: aggregate_report_is_withheld_until_both_post_run_hashes_pass
    #[test]
    fn aggregate_report_is_withheld_until_both_post_run_hashes_pass() {
        let mut fake = FakeController::success();
        fake.digests = VecDeque::from([
            Ok([1; 32]),
            Ok([2; 32]),
            Ok([1; 32]),
            Err(Refusal::KernelOperation {
                operation: "fixture-hash",
                errno: Some(5),
            }),
        ]);
        assert_eq!(
            execute(&mut fake),
            Err(Refusal::KernelOperation {
                operation: "fixture-hash",
                errno: Some(5),
            })
        );
        assert_eq!(fake.detached_ids, [1, 2]);
        assert_eq!(fake.digest_calls, 4);
    }

    #[test]
    fn evidence_gate_never_publishes_before_final_verify_and_detach() {
        let mut gate = EvidenceGate::default();
        gate.observe(512).expect("one pending observation");
        assert_eq!(gate.publish(), Err(Refusal::ProtocolOrder));

        let mut gate = EvidenceGate::default();
        gate.observe(512).expect("one pending observation");
        gate.verify().expect("final identity verified");
        assert_eq!(gate.publish(), Err(Refusal::ProtocolOrder));
    }

    #[test]
    fn discarded_observation_can_never_be_published() {
        let mut gate = EvidenceGate::default();
        gate.observe(512).expect("one pending observation");
        gate.discard().expect("mismatch discards it");
        gate.detached().expect("cleanup completed");
        assert!(gate.prove_discarded().is_ok());
    }
}
