//! [X509 Certificate](https://www.rfc-editor.org/rfc/rfc5280) Implementation

pub mod identity;

use crate::der::{self, BitString, Parse};
pub use identity::Identity;

use sl_std::{ascii, base64, big_num::BigNum, datetime::DateTime};

#[derive(Clone, Debug)]
pub struct X509Certificate {
    pub version: usize,
    pub serial_number: BigNum,
    pub signature_algorithm: AlgorithmIdentifier,
    pub issuer: Identity,
    pub validity: Validity,
}

#[derive(Clone, Debug)]
pub struct SignedCertificate {
    certificate: X509Certificate,
    _signature_algorithm: AlgorithmIdentifier,
    _signature: BitString,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlgorithmIdentifier {
    pub identifier: der::ObjectIdentifier,
}

#[derive(Clone, Copy, Debug)]
pub struct Validity {
    pub not_before: DateTime,
    pub not_after: DateTime,
}

#[derive(Clone, Copy, Debug)]
pub enum Error {
    InvalidFormat,
    ParsingFailed(der::Error),
    TrailingBytes,
}

#[derive(Debug)]
pub enum PemParseError {
    Certificate(Error),
    MalformedPem,
}

macro_rules! expect_next_item {
    ($sequence: expr) => {
        $sequence
            .next()
            .map(|e| e.map_err($crate::certificate::Error::ParsingFailed))
            .ok_or($crate::certificate::Error::InvalidFormat)
            .flatten()
    };
}

macro_rules! expect_type {
    ($item: expr, $expected_type: ident) => {
        if let der::Item::$expected_type(value) = $item {
            Ok(value)
        } else {
            Err($crate::certificate::Error::InvalidFormat)
        }
    };
}

// Export the macros above within the crate (but don't expose the publicly)
pub(crate) use {expect_next_item, expect_type};

impl der::Parse for X509Certificate {
    type Error = Error;

    fn try_from_item(item: der::Item<'_>) -> Result<Self, Self::Error> {
        let mut certificate = expect_type!(item, Sequence)?;

        let version = parse_certificate_version(expect_next_item!(certificate)?)?;

        let serial_number = expect_type!(expect_next_item!(certificate)?, Integer)?.into();

        let signature_algorithm =
            AlgorithmIdentifier::try_from_item(expect_next_item!(certificate)?)?;

        let issuer = Identity::try_from_item(expect_next_item!(certificate)?)?;

        let validity = Validity::try_from_item(expect_next_item!(certificate)?)?;

        Ok(Self {
            version,
            serial_number,
            signature_algorithm,
            issuer,
            validity,
        })
    }
}

impl SignedCertificate {
    pub fn new(bytes: &[u8]) -> Result<Self, Error> {
        // The root sequence always has the following structure:
        // * data
        // * Signature algorithm used
        // * Signature

        let (root_sequence, bytes_consumed) = der::Item::parse(bytes)?;
        let mut root_sequence = expect_type!(root_sequence, Sequence)?;

        if bytes_consumed != bytes.len() {
            return Err(Error::TrailingBytes);
        }

        let certificate = X509Certificate::try_from_item(expect_next_item!(root_sequence)?)?;

        let signature_algorithm =
            AlgorithmIdentifier::try_from_item(expect_next_item!(root_sequence)?)?;

        let _signature = expect_type!(expect_next_item!(root_sequence)?, BitString)?;

        if root_sequence.next().is_some() {
            return Err(Error::InvalidFormat);
        }

        if certificate.signature_algorithm != signature_algorithm {
            log::error!("The signature algorithm specified in the certificate {:?} does not match the algorithm used for the actual signature {:?}", certificate.signature_algorithm, signature_algorithm);
            return Err(Error::InvalidFormat);
        }

        Ok(Self {
            certificate,
            _signature_algorithm: signature_algorithm,
            _signature,
        })
    }

    /// Validates the basic properties of a certificate
    ///
    /// Precisely, we check if the signature on a certificate is *correct* and if the certificate
    /// is valid for the current time. However, we do **not** verify that we trust the issuer
    /// of said certificate!
    pub fn is_valid(&self) -> bool {
        let now = DateTime::now();
        self.certificate.validity.not_before <= now && now <= self.certificate.validity.not_after
    }

    pub fn load_from_pem(data: &[u8]) -> Result<Self, PemParseError> {
        let str: &ascii::Str = data.try_into().map_err(|_| PemParseError::MalformedPem)?;
        let mut lines = str.trim().lines();

        // Throw away the first and last lines (those delimit the b64 data)
        lines.next().ok_or(PemParseError::MalformedPem)?;
        lines.next_back().ok_or(PemParseError::MalformedPem)?;

        let base64_data: ascii::String = lines.collect();
        let certificate_bytes =
            base64::b64decode(&base64_data).map_err(|_| PemParseError::MalformedPem)?;

        let certificate = Self::new(&certificate_bytes)?;

        Ok(certificate)
    }
}

impl From<der::Error> for Error {
    fn from(value: der::Error) -> Self {
        Self::ParsingFailed(value)
    }
}

impl der::Parse for AlgorithmIdentifier {
    type Error = Error;

    fn try_from_item(item: der::Item<'_>) -> Result<Self, Self::Error> {
        let mut sequence = expect_type!(item, Sequence)?;
        let identifier = expect_type!(expect_next_item!(sequence)?, ObjectIdentifier)?;
        let _parameters = expect_next_item!(sequence)?;

        if sequence.next().is_some() {
            return Err(Error::TrailingBytes);
        }
        Ok(Self { identifier })
    }
}

impl der::Parse for Validity {
    type Error = Error;

    fn try_from_item(item: der::Item<'_>) -> Result<Self, Self::Error> {
        let mut sequence = expect_type!(item, Sequence)?;

        let not_before = expect_type!(expect_next_item!(sequence)?, UtcTime)?;
        let not_after = expect_type!(expect_next_item!(sequence)?, UtcTime)?;

        if sequence.next().is_some() {
            return Err(Error::TrailingBytes);
        }

        Ok(Validity {
            not_before,
            not_after,
        })
    }
}

fn parse_certificate_version(item: der::Item<'_>) -> Result<usize, Error> {
    let (version_item, _) = der::Item::parse(expect_type!(item, ContextSpecific)?)?;

    expect_type!(version_item, Integer)?
        .try_into()
        .map_err(|_| Error::InvalidFormat)
}

impl From<SignedCertificate> for X509Certificate {
    fn from(value: SignedCertificate) -> Self {
        value.certificate
    }
}

impl From<Error> for PemParseError {
    fn from(value: Error) -> Self {
        Self::Certificate(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_PEM: &[u8] =
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/cert.pem"));

    #[test]
    fn parse_pem() {
        let _parsed_certificate = SignedCertificate::load_from_pem(EXAMPLE_PEM).unwrap();
    }
}
