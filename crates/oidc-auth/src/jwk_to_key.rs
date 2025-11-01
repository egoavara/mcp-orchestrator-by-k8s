use crate::jwks::{
    LocalAlgorithmParameters, LocalEllipticCurveKeyParameters, LocalJwk,
    LocalOctetKeyPairParameters, LocalRSAKeyParameters,
};
use base64::{Engine, engine::general_purpose};
use jsonwebtoken::jwk::EllipticCurve;
use jsonwebtoken::{DecodingKey, EncodingKey};

pub fn jwk_to_encoding_key(jwk: &LocalJwk) -> Result<EncodingKey, String> {
    match &jwk.algorithm {
        LocalAlgorithmParameters::OctetKeyPair(okp) => okp_to_encoding_key(okp),
        LocalAlgorithmParameters::Rsa(rsa) => rsa_to_encoding_key(rsa),
        LocalAlgorithmParameters::EllipticCurve(ec) => ec_to_encoding_key(ec),
        LocalAlgorithmParameters::OctetKey(oct) => EncodingKey::from_base64_secret(&oct.value)
            .map_err(|e| format!("Failed to create symmetric key: {}", e)),
    }
}

pub fn jwk_to_decoding_key(jwk: &LocalJwk) -> Result<DecodingKey, String> {
    match &jwk.algorithm {
        LocalAlgorithmParameters::OctetKeyPair(okp) => DecodingKey::from_ed_components(&okp.x)
            .map_err(|e| format!("Failed to create OKP decoding key: {}", e)),
        LocalAlgorithmParameters::Rsa(rsa) => DecodingKey::from_rsa_components(&rsa.n, &rsa.e)
            .map_err(|e| format!("Failed to create RSA decoding key: {}", e)),
        LocalAlgorithmParameters::EllipticCurve(ec) => {
            DecodingKey::from_ec_components(&ec.x, &ec.y)
                .map_err(|e| format!("Failed to create EC decoding key: {}", e))
        }
        LocalAlgorithmParameters::OctetKey(oct) => {
            let secret = base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(&oct.value)
                .map_err(|e| format!("Failed to decode symmetric key: {}", e))?;
            Ok(DecodingKey::from_secret(secret.as_slice()))
        }
    }
}
fn okp_to_encoding_key(okp: &LocalOctetKeyPairParameters) -> Result<EncodingKey, String> {
    let d = okp
        .d
        .as_ref()
        .ok_or("Missing private key 'd' in OctetKeyPair")?;

    let key_bytes = general_purpose::URL_SAFE_NO_PAD
        .decode(d)
        .map_err(|e| format!("Failed to decode private key: {}", e))?;

    match &okp.curve {
        EllipticCurve::Ed25519 => {
            let key_array: [u8; 32] = key_bytes
                .try_into()
                .map_err(|_| "Invalid Ed25519 key length, expected 32 bytes")?;

            let signing_key = ed25519_dalek::SigningKey::from_bytes(&key_array);

            let der = pkcs8::EncodePrivateKey::to_pkcs8_der(&signing_key)
                .map_err(|e| format!("Failed to encode Ed25519 to DER: {}", e))?;

            Ok(EncodingKey::from_ed_der(der.as_bytes()))
        }
        curve => Err(format!("Unsupported OKP curve: {:?}", curve)),
    }
}

fn rsa_to_encoding_key(rsa: &LocalRSAKeyParameters) -> Result<EncodingKey, String> {
    let n = decode_base64_bigint(&rsa.n)?;
    let e = decode_base64_bigint(&rsa.e)?;
    let d = rsa
        .d
        .as_ref()
        .ok_or("Missing private exponent 'd' in RSA key")?;
    let d = decode_base64_bigint(d)?;

    let private_key = rsa::RsaPrivateKey::from_components(n, e, d, vec![])
        .map_err(|e| format!("Failed to construct RSA private key: {}", e))?;

    let der = pkcs8::EncodePrivateKey::to_pkcs8_der(&private_key)
        .map_err(|e| format!("Failed to encode RSA to DER: {}", e))?;

    Ok(EncodingKey::from_rsa_der(der.as_bytes()))
}

fn ec_to_encoding_key(ec: &LocalEllipticCurveKeyParameters) -> Result<EncodingKey, String> {
    let d = ec.d.as_ref().ok_or("Missing private key 'd' in EC key")?;

    let d_bytes = general_purpose::URL_SAFE_NO_PAD
        .decode(d)
        .map_err(|e| format!("Failed to decode EC private key: {}", e))?;

    match &ec.curve {
        EllipticCurve::P256 => {
            use p256::SecretKey;

            let secret_key = SecretKey::from_slice(&d_bytes)
                .map_err(|e| format!("Invalid P-256 private key: {}", e))?;

            let der = pkcs8::EncodePrivateKey::to_pkcs8_der(&secret_key)
                .map_err(|e| format!("Failed to encode P-256 to DER: {}", e))?;

            Ok(EncodingKey::from_ec_der(der.as_bytes()))
        }
        EllipticCurve::P384 => {
            use p384::SecretKey;

            let secret_key = SecretKey::from_slice(&d_bytes)
                .map_err(|e| format!("Invalid P-384 private key: {}", e))?;

            let der = pkcs8::EncodePrivateKey::to_pkcs8_der(&secret_key)
                .map_err(|e| format!("Failed to encode P-384 to DER: {}", e))?;

            Ok(EncodingKey::from_ec_der(der.as_bytes()))
        }
        curve => Err(format!(
            "Unsupported EC curve: {:?} (supported: P-256, P-384)",
            curve
        )),
    }
}

fn decode_base64_bigint(s: &str) -> Result<rsa::BigUint, String> {
    let bytes = general_purpose::URL_SAFE_NO_PAD
        .decode(s)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;
    Ok(rsa::BigUint::from_bytes_be(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symmetric_key() {
        let jwk_json = r#"{
            "kty": "oct",
            "k": "AyM1SysPpbyDfgZld3umj1qzKObwVMkoqQ-EstJQLr_T-1qS0gZH75aKtMN3Yj0iPS4hcgUuTwjAzZr1Z9CAow",
            "alg": "HS256"
        }"#;

        let jwk: LocalJwk = serde_json::from_str(jwk_json).unwrap();
        let result = jwk_to_encoding_key(&jwk);
        assert!(result.is_ok());
    }
}
