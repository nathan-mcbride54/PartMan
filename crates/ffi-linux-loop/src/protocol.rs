//! Pure loop-acceptance protocol and evidence-publication gate.
//!
//! Two protocols live here. [`execute`] is increment 2e's two-leg acceptance
//! with its in-process probe. [`execute_session`] is increment 2f's hold-open
//! session: the same configure/verify/detach discipline, but the observation
//! interval contains crate-launched external probes, and every launch is
//! bracketed by node and status verification as control flow rather than as a
//! convention a caller could skip. Captured prober output is quarantined in
//! the session gate and released only by [`SessionEvidenceGate::publish`],
//! which requires confirmed detach first — so a caller cannot hold probe
//! bytes while the loop device is still bound.

use crate::{
    FixtureRole, ProbeRecord, ProbeSubject, ProbeTool, Refusal, RunReport, SessionDiskFacts,
    SessionPartitionFacts, SessionReport,
};

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

/// Most partitions one session will enumerate before refusing. The registered
/// fixtures carry a handful; a larger count means the attached object is not
/// the fixture this session verified, so the bound fails closed rather than
/// probing an unexpected layout.
pub(super) const MAX_SESSION_PARTITIONS: usize = 15;

/// One captured external launch, before the protocol wraps it into the public
/// [`ProbeRecord`]. Raw bytes only; the launching side never interprets them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CapturedProbe {
    pub(super) exit_code: Option<i32>,
    pub(super) stdout: Vec<u8>,
    pub(super) stderr: Vec<u8>,
}

/// Increment 2f's hold-open boundary, mirrored from [`Controller`].
///
/// The launch method receives a subject and tool enumerated by this protocol,
/// never a caller-selected path, descriptor, or argument. Node re-statting is
/// a separate method so the bracket around every launch is visible — and
/// testable — as protocol control flow.
pub(super) trait SessionController {
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

    /// Re-stat the public device node for `subject` by name and require its
    /// identity to equal the held/derived one recorded at configure time.
    fn verify_node(
        &mut self,
        attachment: &Self::Attachment,
        subject: ProbeSubject,
    ) -> Result<(), Refusal>;

    /// Enumerate materialized partition indices from the descriptor-derived
    /// sysfs root. In-process; not an external call.
    fn enumerate_partitions(&mut self, attachment: &Self::Attachment) -> Result<Vec<u32>, Refusal>;

    /// Hash the attached device's complete logical contents through the held
    /// loop descriptor. In-process; not an external call.
    fn device_digest(&mut self, attachment: &Self::Attachment) -> Result<[u8; 32], Refusal>;

    /// Capture the named sysfs projection facts for the disk and every
    /// enumerated partition, and refuse if any session device number appears
    /// in the mount table or any node is writable. In-process.
    fn capture_facts(
        &mut self,
        attachment: &Self::Attachment,
    ) -> Result<(SessionDiskFacts, Vec<SessionPartitionFacts>), Refusal>;

    /// Launch one predeclared external tool against the subject and capture
    /// its bounded output. The crate owns this launch entirely.
    fn launch(
        &mut self,
        attachment: &Self::Attachment,
        subject: ProbeSubject,
        tool: ProbeTool,
    ) -> Result<CapturedProbe, Refusal>;

    fn detach(&mut self, attachment: Self::Attachment) -> Result<(), Refusal>;
}

/// Quarantine for captured prober output.
///
/// Records accumulate here and leave only through [`Self::publish`], which
/// requires the closed sequence, confirmed detach, and at least one record.
/// Every refusal path drops the gate — and the bytes — unpublished.
#[derive(Debug, Default)]
struct SessionEvidenceGate {
    records: Vec<ProbeRecord>,
    facts: Option<(SessionDiskFacts, Vec<SessionPartitionFacts>)>,
    closed: bool,
    detached: bool,
}

impl SessionEvidenceGate {
    fn observe(&mut self, record: ProbeRecord) -> Result<(), Refusal> {
        if self.closed || self.detached {
            return Err(Refusal::ProtocolOrder);
        }
        self.records.push(record);
        Ok(())
    }

    fn close(
        &mut self,
        facts: (SessionDiskFacts, Vec<SessionPartitionFacts>),
    ) -> Result<(), Refusal> {
        if self.closed || self.detached || self.records.is_empty() {
            return Err(Refusal::ProtocolOrder);
        }
        self.closed = true;
        self.facts = Some(facts);
        Ok(())
    }

    fn detached(&mut self) -> Result<(), Refusal> {
        if self.detached {
            return Err(Refusal::ProtocolOrder);
        }
        self.detached = true;
        Ok(())
    }

    fn publish(self, fixture: FixtureRole) -> Result<SessionReport, Refusal> {
        if !self.closed || !self.detached || self.records.is_empty() {
            return Err(Refusal::ProtocolOrder);
        }
        let (disk_facts, partition_facts) = self.facts.ok_or(Refusal::ProtocolOrder)?;
        let partitions_observed =
            u8::try_from(partition_facts.len()).map_err(|_| Refusal::ProtocolOrder)?;
        Ok(SessionReport {
            fixture,
            partitions_observed,
            disk_facts,
            partition_facts,
            records: self.records,
        })
    }
}

/// Run one increment 2f hold-open session over a single authorized fixture.
///
/// Sequence: initial catalogue hash, one read-only configure, verification,
/// the whole-device hash through the held loop descriptor (before the first
/// external launch), a bracketed `udevadm settle`, partition enumeration, the
/// sysfs facts capture with its mount-absence check, then for the disk and
/// each partition the bracketed predeclared probes — the udev query twice for
/// the instrument's byte-stability gate — the second whole-device hash (after
/// the last external launch), closure, unconditional detach with the detach
/// error taking precedence, the post-run fixture hash, and only then
/// publication of the quarantined records.
pub(super) fn execute_session<C: SessionController>(
    controller: &mut C,
    fixture: FixtureRole,
) -> Result<SessionReport, Refusal> {
    let before = controller.digest(fixture)?;
    if before != controller.expected_digest(fixture) {
        return Err(Refusal::InitialFixtureHashMismatch { fixture });
    }

    let attachment = match controller.configure(fixture, ConfigureRequest::READ_ONLY_ACCEPTANCE) {
        Ok(attachment) => Ok(attachment),
        Err(ConfigureError::Busy) => Err(Refusal::LoopIsolationConflict),
        Err(ConfigureError::Refused(error)) => Err(error),
    }?;

    let mut gate = SessionEvidenceGate::default();
    let session_result = (|| {
        controller.verify(&attachment, fixture)?;
        if controller.device_digest(&attachment)? != controller.expected_digest(fixture) {
            return Err(Refusal::LoopDeviceHashMismatch);
        }
        launch_bracketed(
            controller,
            &attachment,
            fixture,
            ProbeSubject::Disk,
            ProbeTool::UdevadmSettle,
            &mut gate,
        )?;
        let partitions = controller.enumerate_partitions(&attachment)?;
        if partitions.len() > MAX_SESSION_PARTITIONS {
            return Err(Refusal::PartitionCountExceeded);
        }
        let facts = controller.capture_facts(&attachment)?;
        if facts.1.len() != partitions.len() {
            return Err(Refusal::ProtocolOrder);
        }
        let mut subjects = Vec::with_capacity(partitions.len() + 1);
        subjects.push(ProbeSubject::Disk);
        subjects.extend(partitions.into_iter().map(ProbeSubject::Partition));
        for subject in subjects {
            for tool in [
                ProbeTool::UdevadmInfo,
                ProbeTool::UdevadmInfo,
                ProbeTool::BlkidProbe,
                ProbeTool::WipefsNoAct,
            ] {
                launch_bracketed(controller, &attachment, fixture, subject, tool, &mut gate)?;
            }
        }
        if controller.device_digest(&attachment)? != controller.expected_digest(fixture) {
            return Err(Refusal::LoopDeviceHashMismatch);
        }
        gate.close(facts)
    })();
    // Exactly 2e's cleanup precedence: detach runs unconditionally, and a
    // cleanup failure overrides an otherwise-publishable session.
    let session_detach = controller.detach(attachment);
    session_detach?;
    gate.detached()?;
    session_result?;

    if controller.digest(fixture)? != before {
        return Err(Refusal::FixtureHashChanged { fixture });
    }

    // Publication is last: the captured bytes stay sealed through every
    // bracket, the confirmed detach and partition teardown inside detach,
    // and the post-run hash. No earlier success object exists.
    gate.publish(fixture)
}

/// The launch bracket, as control flow: node identity and the full status
/// binding are re-verified immediately before and immediately after every
/// external call, exactly as the increment 2f boundary requires. A caller
/// cannot reach `launch` around this function because the trait is private
/// and this is its only protocol call site.
fn launch_bracketed<C: SessionController>(
    controller: &mut C,
    attachment: &C::Attachment,
    fixture: FixtureRole,
    subject: ProbeSubject,
    tool: ProbeTool,
    gate: &mut SessionEvidenceGate,
) -> Result<(), Refusal> {
    controller.verify_node(attachment, subject)?;
    controller.verify(attachment, fixture)?;
    let capture = controller.launch(attachment, subject, tool)?;
    controller.verify_node(attachment, subject)?;
    controller.verify(attachment, fixture)?;
    gate.observe(ProbeRecord {
        tool,
        subject,
        exit_code: capture.exit_code,
        stdout: capture.stdout,
        stderr: capture.stderr,
    })
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

    #[derive(Debug, PartialEq, Eq)]
    enum SessionEvent {
        Digest(FixtureRole),
        Configure(FixtureRole),
        Verify(FixtureRole),
        VerifyNode(ProbeSubject),
        Enumerate,
        DeviceDigest,
        CaptureFacts,
        Launch(ProbeSubject, ProbeTool),
        Detach,
    }

    fn fake_disk_facts() -> SessionDiskFacts {
        SessionDiskFacts {
            size_sectors: 40,
            read_only: true,
            logical_block_size: 512,
        }
    }

    fn fake_partition_facts() -> Vec<SessionPartitionFacts> {
        vec![SessionPartitionFacts {
            index: 1,
            start_sectors: 8,
            size_sectors: 16,
            read_only: true,
        }]
    }

    struct SessionAttachment {
        released: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }

    impl Drop for SessionAttachment {
        fn drop(&mut self) {
            self.released
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    struct FakeSessionController {
        digests: VecDeque<Result<[u8; 32], Refusal>>,
        configure: VecDeque<Result<(), ConfigureError>>,
        verify: VecDeque<Result<(), Refusal>>,
        verify_node: VecDeque<Result<(), Refusal>>,
        enumerate: VecDeque<Result<Vec<u32>, Refusal>>,
        device_digests: VecDeque<Result<[u8; 32], Refusal>>,
        facts: VecDeque<Result<(SessionDiskFacts, Vec<SessionPartitionFacts>), Refusal>>,
        launches: VecDeque<Result<CapturedProbe, Refusal>>,
        detach: VecDeque<Result<(), Refusal>>,
        panic_on_launch: Option<usize>,
        launch_calls: usize,
        digest_calls: usize,
        events: Vec<SessionEvent>,
        released: std::sync::Arc<std::sync::atomic::AtomicBool>,
        expected: [u8; 32],
    }

    impl FakeSessionController {
        /// A scripted success over a disk with exactly one partition: settle
        /// plus four tool launches (the udev query twice, then blkid and
        /// wipefs) against two subjects is nine launches, each bracketed by
        /// two node re-stats and two full verifications, with the whole-device
        /// hash taken before the first and after the last launch.
        fn success() -> Self {
            let capture = CapturedProbe {
                exit_code: Some(0),
                stdout: b"captured".to_vec(),
                stderr: Vec::new(),
            };
            Self {
                digests: VecDeque::from([Ok([7; 32]), Ok([7; 32])]),
                configure: VecDeque::from([Ok(())]),
                verify: VecDeque::from(vec![Ok(()); 19]),
                verify_node: VecDeque::from(vec![Ok(()); 18]),
                enumerate: VecDeque::from([Ok(vec![1])]),
                device_digests: VecDeque::from([Ok([7; 32]), Ok([7; 32])]),
                facts: VecDeque::from([Ok((fake_disk_facts(), fake_partition_facts()))]),
                launches: VecDeque::from(vec![Ok(capture); 9]),
                detach: VecDeque::from([Ok(())]),
                panic_on_launch: None,
                launch_calls: 0,
                digest_calls: 0,
                events: Vec::new(),
                released: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
                expected: [7; 32],
            }
        }
    }

    impl SessionController for FakeSessionController {
        type Attachment = SessionAttachment;

        fn expected_digest(&self, _fixture: FixtureRole) -> [u8; 32] {
            self.expected
        }

        fn digest(&mut self, fixture: FixtureRole) -> Result<[u8; 32], Refusal> {
            self.events.push(SessionEvent::Digest(fixture));
            self.digest_calls += 1;
            self.digests
                .pop_front()
                .expect("fake session digest script is complete")
        }

        fn configure(
            &mut self,
            fixture: FixtureRole,
            request: ConfigureRequest,
        ) -> Result<Self::Attachment, ConfigureError> {
            assert_eq!(request, ConfigureRequest::READ_ONLY_ACCEPTANCE);
            self.events.push(SessionEvent::Configure(fixture));
            self.configure
                .pop_front()
                .expect("fake session configure script is complete")
                .map(|()| SessionAttachment {
                    released: self.released.clone(),
                })
        }

        fn verify(
            &mut self,
            _attachment: &Self::Attachment,
            expected: FixtureRole,
        ) -> Result<(), Refusal> {
            self.events.push(SessionEvent::Verify(expected));
            self.verify
                .pop_front()
                .expect("fake session verify script is complete")
        }

        fn verify_node(
            &mut self,
            _attachment: &Self::Attachment,
            subject: ProbeSubject,
        ) -> Result<(), Refusal> {
            self.events.push(SessionEvent::VerifyNode(subject));
            self.verify_node
                .pop_front()
                .expect("fake session verify-node script is complete")
        }

        fn enumerate_partitions(
            &mut self,
            _attachment: &Self::Attachment,
        ) -> Result<Vec<u32>, Refusal> {
            self.events.push(SessionEvent::Enumerate);
            self.enumerate
                .pop_front()
                .expect("fake session enumerate script is complete")
        }

        fn device_digest(&mut self, _attachment: &Self::Attachment) -> Result<[u8; 32], Refusal> {
            self.events.push(SessionEvent::DeviceDigest);
            self.device_digests
                .pop_front()
                .expect("fake session device-digest script is complete")
        }

        fn capture_facts(
            &mut self,
            _attachment: &Self::Attachment,
        ) -> Result<(SessionDiskFacts, Vec<SessionPartitionFacts>), Refusal> {
            self.events.push(SessionEvent::CaptureFacts);
            self.facts
                .pop_front()
                .expect("fake session facts script is complete")
        }

        fn launch(
            &mut self,
            _attachment: &Self::Attachment,
            subject: ProbeSubject,
            tool: ProbeTool,
        ) -> Result<CapturedProbe, Refusal> {
            self.events.push(SessionEvent::Launch(subject, tool));
            self.launch_calls += 1;
            assert_ne!(
                self.panic_on_launch,
                Some(self.launch_calls),
                "scripted mid-window failure"
            );
            self.launches
                .pop_front()
                .expect("fake session launch script is complete")
        }

        fn detach(&mut self, attachment: Self::Attachment) -> Result<(), Refusal> {
            self.events.push(SessionEvent::Detach);
            drop(attachment);
            self.detach
                .pop_front()
                .expect("fake session detach script is complete")
        }
    }

    // Requirements: SAFE-004, SAFE-005, SAFE-007
    //   Every external launch is bracketed by node and status verification on both
    //   sides, as protocol control flow; publication happens only after closure,
    //   confirmed detach, and the unchanged post-run hash.
    // Evidence: session_brackets_every_launch_and_publishes_only_after_detach_and_hash
    #[test]
    fn session_brackets_every_launch_and_publishes_only_after_detach_and_hash() {
        let mut fake = FakeSessionController::success();
        let report =
            execute_session(&mut fake, FixtureRole::Basic).expect("complete scripted session");
        assert_eq!(report.fixture(), FixtureRole::Basic);
        assert_eq!(report.partitions_observed(), 1);
        assert_eq!(report.records().len(), 9);
        assert!(report.bindings_verified_around_every_launch());
        assert!(report.detachment_confirmed());
        assert!(report.partition_teardown_confirmed());
        assert!(report.initial_fixture_hash_matched_catalogue());
        assert!(report.fixture_hash_unchanged());
        assert!(report.captured_output_quarantined_until_teardown());
        assert!(report.loop_content_hashes_matched_catalogue());
        assert!(report.nodes_unmounted_and_read_only());
        assert_eq!(report.disk_facts(), fake_disk_facts());
        assert_eq!(report.partition_facts(), fake_partition_facts());
        assert_eq!(report.records()[0].tool(), ProbeTool::UdevadmSettle);
        assert_eq!(report.records()[0].subject(), ProbeSubject::Disk);
        assert_eq!(report.records()[0].stdout(), b"captured");
        assert_eq!(report.records()[0].exit_code(), Some(0));
        assert_eq!(report.records()[1].tool(), ProbeTool::UdevadmInfo);
        assert_eq!(report.records()[2].tool(), ProbeTool::UdevadmInfo);
        assert_eq!(report.records()[8].tool(), ProbeTool::WipefsNoAct);
        assert_eq!(report.records()[8].subject(), ProbeSubject::Partition(1));

        let mut expected_events = vec![
            SessionEvent::Digest(FixtureRole::Basic),
            SessionEvent::Configure(FixtureRole::Basic),
            SessionEvent::Verify(FixtureRole::Basic),
            SessionEvent::DeviceDigest,
        ];
        let bracket = |events: &mut Vec<SessionEvent>, subject: ProbeSubject, tool: ProbeTool| {
            events.push(SessionEvent::VerifyNode(subject));
            events.push(SessionEvent::Verify(FixtureRole::Basic));
            events.push(SessionEvent::Launch(subject, tool));
            events.push(SessionEvent::VerifyNode(subject));
            events.push(SessionEvent::Verify(FixtureRole::Basic));
        };
        bracket(
            &mut expected_events,
            ProbeSubject::Disk,
            ProbeTool::UdevadmSettle,
        );
        expected_events.push(SessionEvent::Enumerate);
        expected_events.push(SessionEvent::CaptureFacts);
        for subject in [ProbeSubject::Disk, ProbeSubject::Partition(1)] {
            for tool in [
                ProbeTool::UdevadmInfo,
                ProbeTool::UdevadmInfo,
                ProbeTool::BlkidProbe,
                ProbeTool::WipefsNoAct,
            ] {
                bracket(&mut expected_events, subject, tool);
            }
        }
        expected_events.push(SessionEvent::DeviceDigest);
        expected_events.push(SessionEvent::Detach);
        expected_events.push(SessionEvent::Digest(FixtureRole::Basic));
        assert_eq!(fake.events, expected_events);
    }

    // Requirements: SAFE-001, SAFE-005, SAFE-007
    //   The attached device's logical contents must equal the compiled catalogue
    //   digest before the first and after the last external launch; either
    //   mismatch refuses and still detaches.
    // Evidence: session_device_hash_mismatch_refuses_and_still_detaches
    #[test]
    fn session_device_hash_mismatch_refuses_and_still_detaches() {
        let mut fake = FakeSessionController::success();
        fake.device_digests = VecDeque::from([Ok([9; 32])]);
        assert_eq!(
            execute_session(&mut fake, FixtureRole::Basic),
            Err(Refusal::LoopDeviceHashMismatch)
        );
        assert!(fake.events.contains(&SessionEvent::Detach));
        assert_eq!(fake.launch_calls, 0, "no launch before the first hash");

        let mut fake = FakeSessionController::success();
        fake.device_digests = VecDeque::from([Ok([7; 32]), Ok([9; 32])]);
        assert_eq!(
            execute_session(&mut fake, FixtureRole::Basic),
            Err(Refusal::LoopDeviceHashMismatch)
        );
        assert!(fake.events.contains(&SessionEvent::Detach));
        assert_eq!(fake.launch_calls, 9, "the second hash follows every launch");
    }

    // Requirements: SAFE-005, SAFE-007
    //   A mounted or writable session node refuses before any probe launches
    //   against it, and the attachment is still detached.
    // Evidence: session_mounted_or_writable_node_refuses_and_still_detaches
    #[test]
    fn session_mounted_or_writable_node_refuses_and_still_detaches() {
        for refusal in [Refusal::SessionNodeMounted, Refusal::SessionNodeWritable] {
            let mut fake = FakeSessionController::success();
            fake.facts = VecDeque::from([Err(refusal.clone())]);
            assert_eq!(execute_session(&mut fake, FixtureRole::Basic), Err(refusal));
            assert!(fake.events.contains(&SessionEvent::Detach));
            assert_eq!(fake.launch_calls, 1, "only settle preceded the capture");
        }
    }

    // Requirements: SAFE-001, SAFE-005, SAFE-007
    //   A wrong starting hash refuses before any session loop configuration.
    // Evidence: session_wrong_starting_hash_refuses_before_configure
    #[test]
    fn session_wrong_starting_hash_refuses_before_configure() {
        let mut fake = FakeSessionController::success();
        fake.digests = VecDeque::from([Ok([9; 32])]);
        assert_eq!(
            execute_session(&mut fake, FixtureRole::Conflicting),
            Err(Refusal::InitialFixtureHashMismatch {
                fixture: FixtureRole::Conflicting,
            })
        );
        assert_eq!(fake.events.len(), 1, "no configure and no attachment");
    }

    // Requirements: SAFE-005, SAFE-007
    //   A rebind detected across the open window voids the session, publishes
    //   nothing, and the attachment is still detached.
    // Evidence: session_rebind_detected_across_the_window_voids_and_still_detaches
    #[test]
    fn session_rebind_detected_across_the_window_voids_and_still_detaches() {
        let mut fake = FakeSessionController::success();
        // Initial verify, settle's two brackets, then the post-launch full
        // verification of the first disk probe detects a foreign backing.
        fake.verify = VecDeque::from(vec![
            Ok(()),
            Ok(()),
            Ok(()),
            Ok(()),
            Err(Refusal::BackingIdentityMismatch),
        ]);
        assert_eq!(
            execute_session(&mut fake, FixtureRole::Basic),
            Err(Refusal::BackingIdentityMismatch)
        );
        assert!(fake.events.contains(&SessionEvent::Detach));
        assert_eq!(fake.digest_calls, 1, "no post-run hash may imply success");
    }

    // Requirements: SAFE-004, SAFE-005
    //   A failed launch refuses and the attachment is still detached.
    // Evidence: session_launch_failure_refuses_and_still_detaches
    #[test]
    fn session_launch_failure_refuses_and_still_detaches() {
        let mut fake = FakeSessionController::success();
        fake.launches = VecDeque::from([Err(Refusal::ProbeTimedOut {
            tool: "udevadm-settle",
        })]);
        assert_eq!(
            execute_session(&mut fake, FixtureRole::Basic),
            Err(Refusal::ProbeTimedOut {
                tool: "udevadm-settle",
            })
        );
        assert!(fake.events.contains(&SessionEvent::Detach));
    }

    // Requirements: SAFE-005, SAFE-006, SAFE-007
    //   Captured prober output is unavailable to the caller until detach and
    //   partition teardown are confirmed: a cleanup failure wins over otherwise
    //   publishable records, and the gate refuses publication before detach.
    // Evidence: session_captured_output_is_unavailable_until_detach_is_confirmed
    #[test]
    fn session_captured_output_is_unavailable_until_detach_is_confirmed() {
        let mut fake = FakeSessionController::success();
        fake.detach = VecDeque::from([Err(Refusal::DetachNotConfirmed)]);
        assert_eq!(
            execute_session(&mut fake, FixtureRole::Basic),
            Err(Refusal::DetachNotConfirmed)
        );
        assert_eq!(
            fake.digest_calls, 1,
            "no post-run hash after failed cleanup"
        );

        let mut gate = SessionEvidenceGate::default();
        gate.observe(ProbeRecord {
            tool: ProbeTool::UdevadmSettle,
            subject: ProbeSubject::Disk,
            exit_code: Some(0),
            stdout: b"sealed".to_vec(),
            stderr: Vec::new(),
        })
        .expect("record accumulates in quarantine");
        gate.close((fake_disk_facts(), Vec::new()))
            .expect("sequence closes");
        assert_eq!(
            gate.publish(FixtureRole::Basic),
            Err(Refusal::ProtocolOrder),
            "publication before confirmed detach must refuse"
        );

        let mut gate = SessionEvidenceGate::default();
        gate.observe(ProbeRecord {
            tool: ProbeTool::UdevadmSettle,
            subject: ProbeSubject::Disk,
            exit_code: Some(0),
            stdout: b"sealed".to_vec(),
            stderr: Vec::new(),
        })
        .expect("record accumulates in quarantine");
        gate.close((fake_disk_facts(), Vec::new()))
            .expect("sequence closes");
        gate.detached().expect("cleanup confirmed");
        let report = gate
            .publish(FixtureRole::Basic)
            .expect("post-detach publication succeeds");
        assert_eq!(report.records()[0].stdout(), b"sealed");
    }

    // Requirements: SAFE-005, SAFE-007
    //   A fixture hash change discovered after detach refuses and no report exists.
    // Evidence: session_hash_change_after_detach_refuses
    #[test]
    fn session_hash_change_after_detach_refuses() {
        let mut fake = FakeSessionController::success();
        fake.digests = VecDeque::from([Ok([7; 32]), Ok([8; 32])]);
        assert_eq!(
            execute_session(&mut fake, FixtureRole::Basic),
            Err(Refusal::FixtureHashChanged {
                fixture: FixtureRole::Basic,
            })
        );
        assert!(fake.events.contains(&SessionEvent::Detach));
    }

    // Requirements: SAFE-005, SAFE-007
    //   More materialized partitions than the bound means the attached object is
    //   not the verified fixture; the session refuses and still detaches.
    // Evidence: session_partition_bound_refuses_and_still_detaches
    #[test]
    fn session_partition_bound_refuses_and_still_detaches() {
        let mut fake = FakeSessionController::success();
        fake.enumerate = VecDeque::from([Ok((1..=16).collect())]);
        assert_eq!(
            execute_session(&mut fake, FixtureRole::Basic),
            Err(Refusal::PartitionCountExceeded)
        );
        assert!(fake.events.contains(&SessionEvent::Detach));
    }

    // Requirements: SAFE-005, SAFE-007
    //   A panic inside the open window still releases the held attachment, so
    //   the kernel-side autoclear flag can detach the loop device.
    // Evidence: session_panic_during_a_probe_still_releases_the_attachment
    #[test]
    fn session_panic_during_a_probe_still_releases_the_attachment() {
        let mut fake = FakeSessionController::success();
        fake.panic_on_launch = Some(2);
        let released = fake.released.clone();
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            execute_session(&mut fake, FixtureRole::Basic)
        }));
        assert!(outcome.is_err(), "the scripted panic must propagate");
        assert!(
            released.load(std::sync::atomic::Ordering::SeqCst),
            "unwinding must drop the attachment so autoclear can detach"
        );
    }
}
