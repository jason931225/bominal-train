use anyhow::{Context, Result};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Jwk {
    pub kid: String,
    pub kty: String,
    pub alg: String,
    #[serde(rename = "use")]
    pub use_field: String,
    pub n: String,
    pub e: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SupabaseClaims {
    pub sub: String,
    pub iss: String,
    pub exp: usize,
    #[serde(default)]
    pub iat: Option<usize>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub aud: Option<serde_json::Value>,
}

pub async fn fetch_jwks(client: &Client, jwks_url: &str) -> Result<Jwks> {
    let response = client
        .get(jwks_url)
        .send()
        .await
        .with_context(|| format!("failed to fetch supabase jwks from {jwks_url}"))?
        .error_for_status()
        .context("supabase jwks request failed")?;

    let jwks = response
        .json::<Jwks>()
        .await
        .context("invalid supabase jwks payload")?;

    Ok(jwks)
}

pub fn verify_supabase_jwt(
    token: &str,
    jwks: &Jwks,
    issuer: &str,
    audience: Option<&str>,
) -> Result<SupabaseClaims> {
    let header = decode_header(token).context("unable to decode jwt header")?;
    if header.alg != Algorithm::RS256 {
        anyhow::bail!("unexpected jwt algorithm: {:?}", header.alg);
    }

    let kid = header.kid.as_deref().context("missing jwt kid")?;
    let jwk = jwks
        .keys
        .iter()
        .find(|candidate| candidate.kid == kid)
        .with_context(|| format!("unable to find jwk for kid={kid}"))?;

    let key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
        .context("failed to construct rsa decoding key from jwk")?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[issuer]);
    if let Some(audience) = audience {
        validation.set_audience(&[audience]);
    } else {
        validation.validate_aud = false;
    }

    let claims = decode::<SupabaseClaims>(token, &key, &validation)
        .context("supabase token verification failed")?
        .claims;

    Ok(claims)
}
