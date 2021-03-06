use crate::{
  models::id::{UserId, PasswordResetId},
  utils::HashedPassword,
};

use super::users::User;
use super::super::schema::password_resets;

use chrono::{Utc, Duration, NaiveDateTime};

use sodiumoxide::{
  randombytes,
  crypto::pwhash::{pwhash_verify, HashedPassword as PwhashPassword},
};

use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable, Associations)]
#[belongs_to(User)]
pub struct PasswordReset {
  id: PasswordResetId,
  secret: String,
  expiry: NaiveDateTime,
  user_id: UserId,
}

impl PasswordReset {
  pub fn id(&self) -> PasswordResetId {
    self.id
  }

  pub fn secret(&self) -> &str {
    &self.secret
  }

  pub fn expiry(&self) -> NaiveDateTime {
    self.expiry
  }

  pub fn user_id(&self) -> UserId {
    self.user_id
  }

  pub fn check(&self, bytes: &[u8]) -> bool {
    let mut secret = self.secret.as_bytes().to_vec();
    secret.push(0x00);

    let pw = match PwhashPassword::from_slice(&secret) {
      Some(p) => p,
      None => return false,
    };

    pwhash_verify(&pw, bytes)
  }
}

#[derive(Insertable)]
#[table_name = "password_resets"]
pub struct NewPasswordReset {
  pub id: PasswordResetId,
  secret: String,
  expiry: NaiveDateTime,
  user_id: UserId,
}

impl NewPasswordReset {
  pub fn new(id: PasswordResetId, secret: String, expiry: NaiveDateTime, user_id: UserId) -> Self {
    NewPasswordReset {
      id,
      secret,
      expiry,
      user_id,
    }
  }

  pub fn generate(user_id: UserId) -> (Self, Vec<u8>) {
    let id = PasswordResetId(Uuid::new_v4());
    let expiry = Utc::now().naive_utc() + Duration::days(1);
    let (hashed, secret) = NewPasswordReset::generate_secret();

    (
      NewPasswordReset::new(id, hashed, expiry, user_id),
      secret,
    )
  }

  fn generate_secret() -> (String, Vec<u8>) {
    let bytes = randombytes::randombytes(64);
    let hashed = HashedPassword::from(&bytes);
    (hashed.into_string(), bytes)
  }
}
