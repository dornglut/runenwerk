/// Product/session metadata projected after RunenNet admission.
///
/// RunenNet owns participant membership and connection authorization. These lobby, roster, and
/// settings fields are Runenwerk/product data and deliberately remain outside RunenNet Core.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuthoritativeJoinState {
    pub lobby_id: Option<String>,
    pub roster_player_codes: Vec<String>,
    pub max_players: u8,
    pub ai_fill_target: u8,
    pub settings_json: Option<String>,
}
