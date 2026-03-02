use anyhow::{Context, Result};
use jsonwebtoken::{Algorithm, decode_header};
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

pub fn validate_jwt_header(token: &str) -> Result<()> {
    let header = decode_header(token).context("unable to decode jwt header")?;
    if header.alg != Algorithm::RS256 {
        anyhow::bail!("unexpected jwt algorithm: {:?}", header.alg);
    }
    Ok(())
}
