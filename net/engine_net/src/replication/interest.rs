use runen_net::identity::ConnectionHandle;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InterestPolicy {
    Global,
    OwnerOnly,
    Spatial,
    Team,
    Distance,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct InterestContext {
    pub viewer: ConnectionHandle,
    pub owner: Option<ConnectionHandle>,
    pub same_team: bool,
    pub within_distance: bool,
    pub in_spatial_aoi: bool,
}

pub fn allows_replication(policy: InterestPolicy, ctx: InterestContext) -> bool {
    match policy {
        InterestPolicy::Global => true,
        InterestPolicy::OwnerOnly => ctx.owner == Some(ctx.viewer),
        InterestPolicy::Spatial => ctx.in_spatial_aoi,
        InterestPolicy::Team => ctx.same_team,
        InterestPolicy::Distance => ctx.within_distance,
    }
}

#[cfg(test)]
mod tests {
    use super::{InterestContext, InterestPolicy, allows_replication};
    use runen_net::identity::ConnectionHandle;

    #[test]
    fn owner_only_blocks_non_owner_connections() {
        let owner = ConnectionHandle::new(5);
        let viewer = ConnectionHandle::new(8);
        assert!(!allows_replication(
            InterestPolicy::OwnerOnly,
            InterestContext {
                viewer,
                owner: Some(owner),
                same_team: false,
                within_distance: false,
                in_spatial_aoi: false,
            }
        ));
    }

    #[test]
    fn global_allows_every_viewer() {
        assert!(allows_replication(
            InterestPolicy::Global,
            InterestContext {
                viewer: ConnectionHandle::new(1),
                owner: None,
                same_team: false,
                within_distance: false,
                in_spatial_aoi: false,
            }
        ));
    }
}
