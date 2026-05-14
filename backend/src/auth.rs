use crate::{config::Config, error::AppError};
use axum::http::{header, HeaderMap};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionClaims {
    pub sub: String,
    pub login: String,
    pub avatar_url: Option<String>,
    pub exp: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubUser {
    pub id: i64,
    pub login: String,
    pub avatar_url: Option<String>,
}

#[derive(Clone)]
pub struct AuthService {
    config: Arc<Config>,
    states: Arc<Mutex<HashMap<String, chrono::DateTime<Utc>>>>,
}

impl AuthService {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config,
            states: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn new_oauth_state(&self) -> String {
        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
        self.states.lock().await.insert(token.clone(), Utc::now());
        token
    }

    pub async fn verify_oauth_state(&self, state: &str) -> Result<(), AppError> {
        let mut states = self.states.lock().await;
        let created_at = states.remove(state).ok_or(AppError::Unauthorized)?;
        if Utc::now() - created_at > Duration::minutes(10) {
            return Err(AppError::Unauthorized);
        }
        Ok(())
    }

    pub fn is_user_allowed(&self, login: &str) -> Result<(), AppError> {
        let normalized = login.to_lowercase();
        if self.config.github_blocked_users.contains(&normalized) {
            return Err(AppError::Forbidden("your GitHub account is blocked".to_string()));
        }
        if !self.config.github_allowed_users.is_empty()
            && !self.config.github_allowed_users.contains(&normalized)
        {
            return Err(AppError::Forbidden(
                "your GitHub account is not in the allow list".to_string(),
            ));
        }
        Ok(())
    }

    pub fn encode_session(&self, user: &GithubUser) -> Result<String, AppError> {
        let claims = SessionClaims {
            sub: user.id.to_string(),
            login: user.login.clone(),
            avatar_url: user.avatar_url.clone(),
            exp: (Utc::now() + Duration::days(14)).timestamp() as usize,
        };
        Ok(encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.session_secret.as_bytes()),
        )?)
    }

    pub fn decode_session(&self, token: &str) -> Result<SessionClaims, AppError> {
        let data = decode::<SessionClaims>(
            token,
            &DecodingKey::from_secret(self.config.session_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )?;
        Ok(data.claims)
    }

    pub fn build_state_cookie(&self, state: &str) -> String {
        let state_hash = self.hash_state(state);
        format!(
            "oauth_state={state_hash}; HttpOnly; Path=/; Max-Age=600; SameSite=Lax"
        )
    }

    pub fn build_session_cookie(&self, token: &str, secure: bool) -> String {
        let secure_flag = if secure { "; Secure" } else { "" };
        format!(
            "session={token}; HttpOnly; Path=/; Max-Age={}; SameSite=Lax{}",
            60 * 60 * 24 * 14,
            secure_flag
        )
    }

    pub fn build_clear_session_cookie(&self, secure: bool) -> String {
        let secure_flag = if secure { "; Secure" } else { "" };
        format!("session=; HttpOnly; Path=/; Max-Age=0; SameSite=Lax{}", secure_flag)
    }

    pub fn verify_state_cookie(&self, headers: &HeaderMap, state: &str) -> Result<(), AppError> {
        let expected = self.hash_state(state);
        let cookie_header = headers
            .get(header::COOKIE)
            .and_then(|value| value.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let state_cookie = cookie_header
            .split(';')
            .find_map(|part| {
                let trimmed = part.trim();
                trimmed
                    .strip_prefix("oauth_state=")
                    .map(|value| value.to_string())
            })
            .ok_or(AppError::Unauthorized)?;

        if state_cookie != expected {
            return Err(AppError::Unauthorized);
        }
        Ok(())
    }

    fn hash_state(&self, state: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.config.session_secret.as_bytes());
        hasher.update(state.as_bytes());
        URL_SAFE_NO_PAD.encode(hasher.finalize())
    }
}

#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub github_id: i64,
    pub login: String,
    pub avatar_url: Option<String>,
}

pub fn authenticated_user_from_headers(
    auth: &AuthService,
    headers: &HeaderMap,
) -> Result<AuthenticatedUser, AppError> {
    let cookie_header = headers
            .get(header::COOKIE)
            .and_then(|value| value.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

    let token = cookie_header
        .split(';')
        .find_map(|part| part.trim().strip_prefix("session="))
        .ok_or(AppError::Unauthorized)?;

    let claims = auth.decode_session(token)?;
    let github_id = claims.sub.parse().map_err(|_| AppError::Unauthorized)?;
    Ok(AuthenticatedUser {
        github_id,
        login: claims.login,
        avatar_url: claims.avatar_url,
    })
}
