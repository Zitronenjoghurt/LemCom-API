use crate::api::models::query_models::UserSettingsEdit;
use axum::extract::Query;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// User configuration
#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserSettings {
    /// If other people are able to see when you joined the network
    pub join_date_public: bool,
    /// If other people are able to see when you were last online
    pub online_date_public: bool,
}

impl UserSettings {
    pub fn update(&mut self, data: Query<UserSettingsEdit>) {
        self.join_date_public = data.show_join_date.unwrap_or(self.join_date_public);
        self.online_date_public = data.show_online.unwrap_or(self.online_date_public);
    }
}

impl Default for UserSettings {
    fn default() -> Self {
        UserSettings {
            join_date_public: true,
            online_date_public: true,
        }
    }
}
