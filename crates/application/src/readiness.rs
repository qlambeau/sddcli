use domain::{ImplementationPacketRef, SpecReadyDecision, evaluate_spec_ready};

use crate::{ArtifactIdentitySource, ReadinessError};

/// Carries one validated implementation packet reference to the readiness use case.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvaluateSpecReadyCommand {
    packet: ImplementationPacketRef,
}

impl EvaluateSpecReadyCommand {
    /// Creates a readiness command for one packet.
    #[must_use]
    pub const fn new(packet: ImplementationPacketRef) -> Self {
        Self { packet }
    }

    /// Returns the requested packet reference.
    #[must_use]
    pub const fn packet(&self) -> &ImplementationPacketRef {
        &self.packet
    }
}

/// Evaluates one implementation packet through an injected read-only source.
pub struct SpecReadyEvaluator<S> {
    source: S,
}

impl<S> SpecReadyEvaluator<S>
where
    S: ArtifactIdentitySource,
{
    /// Creates a readiness evaluator with an injected artifact source.
    #[must_use]
    pub const fn new(source: S) -> Self {
        Self { source }
    }

    /// Evaluates the requested packet after one repository discovery pass.
    ///
    /// # Errors
    ///
    /// Returns a typed discovery error when the source cannot be read, or
    /// [`ReadinessError::PacketNotFound`] when the packet root is absent.
    pub fn evaluate(
        &self,
        command: &EvaluateSpecReadyCommand,
    ) -> Result<SpecReadyDecision, ReadinessError> {
        let candidates = self.source.discover_identities().map_err(ReadinessError::from)?;
        if !contains_packet_root(&candidates, command.packet()) {
            return Err(ReadinessError::PacketNotFound(command.packet().path().clone()));
        }
        let snapshots = candidates
            .iter()
            .filter_map(|candidate| candidate.snapshot().cloned())
            .collect::<Vec<_>>();
        let decision = evaluate_spec_ready(command.packet(), &snapshots);
        let source_diagnostics = candidates
            .iter()
            .filter(|candidate| {
                decision.result().scope().iter().any(|path| path == candidate.path())
            })
            .filter_map(|candidate| candidate.diagnostic().cloned())
            .collect::<Vec<_>>();
        Ok(decision.with_diagnostics(source_diagnostics))
    }
}

fn contains_packet_root(
    candidates: &[crate::ArtifactCandidate],
    packet: &ImplementationPacketRef,
) -> bool {
    let paths = packet.colocated_paths();
    candidates.iter().any(|candidate| paths.iter().any(|path| path == candidate.path()))
}
