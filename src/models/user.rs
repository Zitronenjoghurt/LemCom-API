use futures::TryStreamExt;
use mongodb::{
    bson::{self, doc},
    options::{FindOptions, UpdateOptions},
    Collection,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::api::models::response_models::{UserPrivateInformation, UserPublicInformation};
use crate::api::utils::time_operations::{nanos_to_date, timestamp_now_nanos};

use super::{enums::PermissionLevel, response_models::Pagination, user_settings::UserSettings};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub key: String,
    pub name: String,
    pub display_name: String,
    pub created_stamp: u64,
    #[serde(default)]
    pub last_access_stamp: u64,
    #[serde(default)]
    pub endpoint_usage: HashMap<String, u64>,
    #[serde(default)]
    pub settings: UserSettings,
    #[serde(default)]
    pub permission_level: PermissionLevel,
}

impl User {
    pub async fn save(&self, collection: &Collection<User>) -> mongodb::error::Result<()> {
        let filter = doc! { "key": &self.key };
        let update = doc! { "$set": bson::to_bson(self)? };
        let options = UpdateOptions::builder().upsert(true).build();

        collection.update_one(filter, update, Some(options)).await?;
        Ok(())
    }

    pub fn use_endpoint(&mut self, method: &str, path: &str) {
        self.last_access_stamp = timestamp_now_nanos();
        *self
            .endpoint_usage
            .entry(format!("{method} {path}"))
            .or_insert(0) += 1;
    }

    pub fn request_count(&self) -> u64 {
        self.endpoint_usage.values().sum()
    }

    pub fn private_information(&self) -> UserPrivateInformation {
        UserPrivateInformation {
            name: self.name.clone(),
            display_name: self.display_name.clone(),
            joined_date: nanos_to_date(self.created_stamp),
            last_online_date: nanos_to_date(self.last_access_stamp),
            total_request_count: self.request_count(),
            permission_level: self.permission_level.clone(),
        }
    }

    pub fn public_information(&self, is_friend: bool) -> UserPublicInformation {
        let joined_date = if self.settings.show_join_date.is_visible(is_friend) {
            Some(nanos_to_date(self.created_stamp))
        } else {
            None
        };
        let last_online_date = if self.settings.show_online_date.is_visible(is_friend) {
            Some(nanos_to_date(self.last_access_stamp))
        } else {
            None
        };
        UserPublicInformation {
            name: self.name.clone(),
            display_name: self.display_name.clone(),
            joined_date,
            last_online_date,
            permission_level: self.permission_level.clone(),
        }
    }
}

pub async fn find_user_by_key(
    collection: &Collection<User>,
    key: &str,
) -> mongodb::error::Result<Option<User>> {
    let filter = doc! { "key": key };
    let user = collection.find_one(Some(filter), None).await?;
    Ok(user)
}

pub async fn find_user_by_name(
    collection: &Collection<User>,
    name: &str,
) -> mongodb::error::Result<Option<User>> {
    let filter = doc! { "name": name.to_lowercase() };
    let user = collection.find_one(Some(filter), None).await?;
    Ok(user)
}

pub async fn get_public_users(
    collection: &Collection<User>,
    page: u32,
    page_size: u32,
) -> mongodb::error::Result<(Vec<User>, Pagination)> {
    let skip = (page - 1) * page_size;
    let find_options = FindOptions::builder()
        .skip(skip as u64)
        .limit(page_size as i64)
        .build();

    let filter = doc! { "settings.show_profile": "Public" };
    let mut cursor = collection.find(filter.clone(), find_options).await?;

    let mut users = Vec::new();
    while let Some(user) = cursor.try_next().await? {
        users.push(user);
    }

    let total: u32 = collection.count_documents(filter, None).await? as u32;
    let pagination = Pagination::new(total, page, page_size, users.len() as u32);

    Ok((users, pagination))
}
