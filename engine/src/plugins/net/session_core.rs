use runen_net::identity::{ConnectionHandle, ParticipantId};
use runen_net::protocol::{NegotiationManager, NegotiationManagerError};
use runen_net::session::{
    ConnectionLossOutcome, MembershipState, RetentionPolicy, Session, SessionError,
};

/// Engine-owned placement of the accepted RunenNet negotiation and session owners.
///
/// This resource does not redefine networking lifecycle semantics. RunenNet remains
/// authoritative for compatibility negotiation, participant membership, connection binding,
/// loss, retention, replacement, and closure. Runenwerk owns only where these Core owners live
/// and when application lifecycle code invokes them.
#[derive(Debug, ecs::Component, ecs::Resource)]
pub struct RunenNetSessionCore {
    negotiation: NegotiationManager,
    session: Session,
}

impl RunenNetSessionCore {
    pub fn new(negotiation: NegotiationManager, session: Session) -> Self {
        Self {
            negotiation,
            session,
        }
    }

    pub fn negotiation(&self) -> &NegotiationManager {
        &self.negotiation
    }

    pub fn negotiation_mut(&mut self) -> &mut NegotiationManager {
        &mut self.negotiation
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn admit_established(
        &mut self,
        participant: ParticipantId,
        connection: ConnectionHandle,
    ) -> Result<(), RunenNetSessionCoreError> {
        let established = self
            .negotiation
            .established(connection)
            .map_err(RunenNetSessionCoreError::Negotiation)?;
        self.session
            .admit_new(participant, established)
            .map_err(RunenNetSessionCoreError::Session)
    }

    pub fn bind_replacement(
        &mut self,
        participant: ParticipantId,
        connection: ConnectionHandle,
    ) -> Result<(), RunenNetSessionCoreError> {
        let established = self
            .negotiation
            .established(connection)
            .map_err(RunenNetSessionCoreError::Negotiation)?;
        self.session
            .bind_replacement(participant, established)
            .map_err(RunenNetSessionCoreError::Session)
    }

    pub fn connection_lost(
        &mut self,
        participant: ParticipantId,
        connection: ConnectionHandle,
        policy: RetentionPolicy,
    ) -> Result<ConnectionLossOutcome, RunenNetSessionCoreError> {
        self.session
            .connection_lost(participant, connection, policy)
            .map_err(RunenNetSessionCoreError::Session)
    }

    pub fn participant_for_connection(
        &self,
        connection: ConnectionHandle,
    ) -> Option<ParticipantId> {
        self.session.participant_for_connection(connection)
    }

    pub fn membership_state(&self, participant: ParticipantId) -> Option<MembershipState> {
        self.session.membership_state(participant)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RunenNetSessionCoreError {
    Negotiation(NegotiationManagerError),
    Session(SessionError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use runen_net::identity::SessionId;
    use runen_net::protocol::{
        CompatibilityOffer, NegotiatedContract, NegotiationManagerLimits, NegotiationRequirements,
        NegotiationStatus, OfferLimits, ProtocolContract, ProtocolId, ProtocolRevision,
    };
    use runen_net::session::{RecoveryDuration, SessionLimits};
    use std::num::{NonZeroU64, NonZeroUsize};

    fn protocol() -> ProtocolContract {
        ProtocolContract::new(ProtocolId::new(1), ProtocolRevision::new(1))
    }

    fn offer() -> CompatibilityOffer {
        CompatibilityOffer::new(vec![protocol()], vec![], vec![], None)
    }

    fn negotiation_manager() -> NegotiationManager {
        NegotiationManager::new(OfferLimits::default(), NegotiationManagerLimits::default())
            .expect("default RunenNet negotiation limits must be valid")
    }

    fn session() -> Session {
        let limit = NonZeroUsize::new(4).expect("non-zero session limit");
        let limits = SessionLimits::new(limit, limit).expect("matching session limits must be valid");
        Session::new(SessionId::new(1), limits)
    }

    fn establish(manager: &mut NegotiationManager, connection: ConnectionHandle) {
        manager
            .start(connection, offer(), offer())
            .expect("compatible negotiation must start");
        manager
            .propose(
                connection,
                NegotiatedContract::new(protocol()),
                &NegotiationRequirements::default(),
            )
            .expect("compatible contract must be proposed");
        assert_ne!(
            manager
                .validate_authority(connection)
                .expect("authority validation must succeed"),
            NegotiationStatus::Established
        );
        assert_eq!(
            manager
                .validate_peer(connection)
                .expect("peer validation must succeed"),
            NegotiationStatus::Established
        );
    }

    #[test]
    fn admission_requires_runennet_established_negotiation() {
        let mut core = RunenNetSessionCore::new(negotiation_manager(), session());
        let participant = ParticipantId::new(1);
        let connection = ConnectionHandle::new(1);

        assert_eq!(
            core.admit_established(participant, connection),
            Err(RunenNetSessionCoreError::Negotiation(
                NegotiationManagerError::UnknownConnection,
            ))
        );
        assert_eq!(core.session().live_memberships(), 0);
    }

    #[test]
    fn terminal_connection_loss_is_owned_by_runennet_session() {
        let connection = ConnectionHandle::new(1);
        let participant = ParticipantId::new(1);
        let mut negotiation = negotiation_manager();
        establish(&mut negotiation, connection);
        let mut core = RunenNetSessionCore::new(negotiation, session());

        core.admit_established(participant, connection)
            .expect("established connection must be admitted");
        assert_eq!(core.participant_for_connection(connection), Some(participant));

        assert_eq!(
            core.connection_lost(participant, connection, RetentionPolicy::Terminate),
            Ok(ConnectionLossOutcome::Terminated)
        );
        assert_eq!(core.participant_for_connection(connection), None);
        assert_eq!(core.membership_state(participant), None);
    }

    #[test]
    fn retained_membership_rebinds_only_to_another_established_connection() {
        let old_connection = ConnectionHandle::new(1);
        let new_connection = ConnectionHandle::new(2);
        let participant = ParticipantId::new(1);
        let mut negotiation = negotiation_manager();
        establish(&mut negotiation, old_connection);
        establish(&mut negotiation, new_connection);
        let mut core = RunenNetSessionCore::new(negotiation, session());

        core.admit_established(participant, old_connection)
            .expect("old connection must be admitted");
        let duration = RecoveryDuration::new(NonZeroU64::new(1).expect("non-zero recovery span"));
        assert!(matches!(
            core.connection_lost(
                participant,
                old_connection,
                RetentionPolicy::RetainForRecovery { duration },
            ),
            Ok(ConnectionLossOutcome::Retained { .. })
        ));

        core.bind_replacement(participant, new_connection)
            .expect("new established connection must rebind retained membership");
        assert_eq!(core.participant_for_connection(old_connection), None);
        assert_eq!(core.participant_for_connection(new_connection), Some(participant));
        assert_eq!(
            core.membership_state(participant),
            Some(MembershipState::Bound(new_connection))
        );
    }
}
