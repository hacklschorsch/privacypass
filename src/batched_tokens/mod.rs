pub mod client;
pub mod server;

use std::io::Write;
use thiserror::*;
use tls_codec::{Deserialize, Serialize, Size, TlsVecU16};
use typenum::U64;
pub use voprf::*;

use crate::{auth::authorize::Token, Nonce, TokenType};

pub type BatchedToken = Token<U64>;

pub type PublicKey = <Ristretto255 as Group>::Elem;

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("Invalid serialized data")]
    InvalidData,
}

// struct {
//     uint8_t blinded_element[Ne];
// } BlindedElement;

pub struct BlindedElement {
    blinded_element: [u8; 32],
}

// struct {
//     uint16_t token_type = 0xF91A;
//     uint8_t token_key_id;
//     BlindedElement blinded_element[Nr];
// } TokenRequest;

pub struct TokenRequest {
    token_type: TokenType,
    token_key_id: u8,
    blinded_elements: TlsVecU16<BlindedElement>,
}

impl TokenRequest {
    /// Returns the number of blinded elements
    pub fn nr(&self) -> usize {
        self.blinded_elements.len()
    }
}

// struct {
//     uint8_t evaluated_element[Ne];
// } EvaluatedElement;

pub struct EvaluatedElement {
    evaluated_element: [u8; 32],
}

// struct {
//     EvaluatedElement evaluated_elements[Nr];
//     uint8_t evaluated_proof[Ns + Ns];
//  } TokenResponse;

pub struct TokenResponse {
    evaluated_elements: TlsVecU16<EvaluatedElement>,
    evaluated_proof: [u8; 64],
}

impl TokenResponse {
    /// Create a new TokenResponse from a byte slice.
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, SerializationError> {
        let mut bytes = bytes;
        Self::tls_deserialize(&mut bytes).map_err(|_| SerializationError::InvalidData)
    }
}

// === TLS codecs ===

impl Size for BlindedElement {
    fn tls_serialized_len(&self) -> usize {
        32
    }
}

impl Serialize for BlindedElement {
    fn tls_serialize<W: Write>(
        &self,
        writer: &mut W,
    ) -> std::result::Result<usize, tls_codec::Error> {
        Ok(writer.write(&self.blinded_element)?)
    }
}

impl Deserialize for BlindedElement {
    fn tls_deserialize<R: std::io::Read>(
        bytes: &mut R,
    ) -> std::result::Result<BlindedElement, tls_codec::Error>
    where
        Self: Sized,
    {
        let mut blinded_element = [0u8; 32];
        bytes.read_exact(&mut blinded_element)?;
        Ok(BlindedElement { blinded_element })
    }
}

impl Size for EvaluatedElement {
    fn tls_serialized_len(&self) -> usize {
        32
    }
}

impl Serialize for EvaluatedElement {
    fn tls_serialize<W: Write>(
        &self,
        writer: &mut W,
    ) -> std::result::Result<usize, tls_codec::Error> {
        Ok(writer.write(&self.evaluated_element)?)
    }
}

impl Deserialize for EvaluatedElement {
    fn tls_deserialize<R: std::io::Read>(
        bytes: &mut R,
    ) -> std::result::Result<EvaluatedElement, tls_codec::Error>
    where
        Self: Sized,
    {
        let mut evaluated_element = [0u8; 32];
        bytes.read_exact(&mut evaluated_element)?;
        Ok(EvaluatedElement { evaluated_element })
    }
}

impl Size for TokenRequest {
    fn tls_serialized_len(&self) -> usize {
        self.token_type.tls_serialized_len()
            + self.token_key_id.tls_serialized_len()
            + self
                .blinded_elements
                .iter()
                .map(|x| x.tls_serialized_len())
                .sum::<usize>()
    }
}

impl Serialize for TokenRequest {
    fn tls_serialize<W: Write>(
        &self,
        writer: &mut W,
    ) -> std::result::Result<usize, tls_codec::Error> {
        Ok(self.token_type.tls_serialize(writer)?
            + self.token_key_id.tls_serialize(writer)?
            + self.blinded_elements.tls_serialize(writer)?)
    }
}

impl Deserialize for TokenRequest {
    fn tls_deserialize<R: std::io::Read>(
        bytes: &mut R,
    ) -> std::result::Result<TokenRequest, tls_codec::Error>
    where
        Self: Sized,
    {
        let token_type = TokenType::tls_deserialize(bytes)?;
        let token_key_id = u8::tls_deserialize(bytes)?;
        let blinded_elements = TlsVecU16::tls_deserialize(bytes)?;

        Ok(TokenRequest {
            token_type,
            token_key_id,
            blinded_elements,
        })
    }
}

impl Size for TokenResponse {
    fn tls_serialized_len(&self) -> usize {
        self.evaluated_elements.tls_serialized_len() + self.evaluated_proof.tls_serialized_len()
    }
}

impl Serialize for TokenResponse {
    fn tls_serialize<W: Write>(
        &self,
        writer: &mut W,
    ) -> std::result::Result<usize, tls_codec::Error> {
        Ok(self.evaluated_elements.tls_serialize(writer)?
            + self.evaluated_proof.tls_serialize(writer)?)
    }
}

impl Deserialize for TokenResponse {
    fn tls_deserialize<R: std::io::Read>(
        bytes: &mut R,
    ) -> std::result::Result<TokenResponse, tls_codec::Error>
    where
        Self: Sized,
    {
        let evaluated_elements = TlsVecU16::tls_deserialize(bytes)?;
        let mut evaluated_proof = [0u8; 64];
        bytes.read_exact(&mut evaluated_proof)?;
        Ok(TokenResponse {
            evaluated_elements,
            evaluated_proof,
        })
    }
}