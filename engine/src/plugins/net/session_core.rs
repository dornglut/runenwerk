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
