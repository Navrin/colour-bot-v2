use colours::models::ParsedColour;
use db::models::Colour;

#[derive(GraphQLObject, Serialize, Deserialize, Debug)]
pub struct ColourResponse {
    pub id: String,
    pub name: String,
    pub colour: String,
}

impl ColourResponse {
    pub fn new_from(model: &Colour, parsed: &ParsedColour) -> Self {
        Self {
            id: model.id.to_string(),
            name: model.name.clone(),
            colour: format!("{}", parsed),
        }
    }
}

#[derive(GraphQLObject, Serialize, Deserialize, Debug)]
pub struct ColourDeleteResponse {
    pub success: bool,
    pub id: String,
}

#[derive(GraphQLInputObject)]
pub struct ColourUpdateInput {
    pub name: Option<String>,
    pub hex: Option<String>,
    pub role_id: Option<String>,
    #[graphql(default)]
    pub update_role_name: bool,
}

#[derive(GraphQLObject, Deserialize, Debug)]
pub struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i32,
    refresh_token: String,
    scope: String,
}

#[derive(Clone, Serialize, Deserialize, GraphQLObject)]
pub struct GuildInfo {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub owner: bool,
    pub permissions: i32,
    #[serde(default)]
    pub cached: bool,
}
