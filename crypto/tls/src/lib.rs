//! TLS 1.2 [RFC 5246](https://www.rfc-editor.org/rfc/rfc5246) implementation.

pub mod alert;
pub mod certificate;
mod connection;
pub mod handshake;
pub mod random;
pub mod record_layer;

mod cipher_suite;
pub use cipher_suite::CipherSuite;
pub use connection::TLSConnection;
