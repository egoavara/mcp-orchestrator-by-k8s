use jsonwebtoken::jwk::{
    CommonParameters, EllipticCurve, EllipticCurveKeyType, OctetKeyPairType, OctetKeyParameters,
    RSAKeyType,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub(crate) struct LocalJwkSet {
    pub keys: Vec<LocalJwk>,
}

/// Parameters for an Elliptic Curve Key
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Default, Hash)]
pub struct LocalEllipticCurveKeyParameters {
    /// Key type value for an Elliptic Curve Key.
    #[serde(rename = "kty")]
    pub key_type: EllipticCurveKeyType,
    /// The "crv" (curve) parameter identifies the cryptographic curve used
    /// with the key.
    #[serde(rename = "crv")]
    pub curve: EllipticCurve,
    /// The "x" (x coordinate) parameter contains the x coordinate for the
    /// Elliptic Curve point.
    pub x: String,
    /// The "y" (y coordinate) parameter contains the y coordinate for the
    /// Elliptic Curve point.
    pub y: String,
    /// The "d" (private exponent) parameter contains the private exponent for the
    /// Elliptic Curve key.
    /// This is optional and only present for private keys.
    #[serde(skip_serializing)]
    pub d: Option<String>,
}

/// Parameters for a RSA Key
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Default, Hash)]
pub struct LocalRSAKeyParameters {
    /// Key type value for a RSA Key
    #[serde(rename = "kty")]
    pub key_type: RSAKeyType,

    /// The "n" (modulus) parameter contains the modulus value for the RSA
    /// public key.
    pub n: String,

    /// The "e" (exponent) parameter contains the exponent value for the RSA
    /// public key.
    pub e: String,

    /// The "d" (private exponent) parameter contains the private exponent for the RSA key.
    /// This is optional and only present for private keys.
    #[serde(skip_serializing)]
    pub d: Option<String>,
}

/// Parameters for an Octet Key Pair
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Default, Hash)]
pub struct LocalOctetKeyPairParameters {
    /// Key type value for an Octet Key Pair
    #[serde(rename = "kty")]
    pub key_type: OctetKeyPairType,
    /// The "crv" (curve) parameter identifies the cryptographic curve used
    /// with the key.
    #[serde(rename = "crv")]
    pub curve: EllipticCurve,
    /// The "x" parameter contains the base64 encoded public key
    pub x: String,

    /// The "d" (private exponent) parameter contains the private exponent for the Octet Key Pair.
    /// This is optional and only present for private keys.
    #[serde(skip_serializing)]
    pub d: Option<String>,
}

/// Algorithm specific parameters
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
#[serde(untagged)]
pub enum LocalAlgorithmParameters {
    EllipticCurve(LocalEllipticCurveKeyParameters),
    RSA(LocalRSAKeyParameters),
    OctetKey(OctetKeyParameters),
    OctetKeyPair(LocalOctetKeyPairParameters),
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct LocalJwk {
    #[serde(flatten)]
    pub common: CommonParameters,
    /// Key algorithm specific parameters
    #[serde(flatten)]
    pub algorithm: LocalAlgorithmParameters,
}
