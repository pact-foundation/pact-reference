//! C FFI friendly version of pact_matching::models::PactSpecification

use pact_models::PactSpecification as NonCPactSpecification;

/// Enum defining the pact specification versions supported by the library
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum PactSpecification {
    /// Unknown or unsupported specification version
    Unknown,
    /// First version of the pact specification (<https://github.com/pact-foundation/pact-specification/tree/version-1>)
    V1,
    /// Second version of the pact specification (<https://github.com/pact-foundation/pact-specification/tree/version-1.1>)
    V1_1,
    /// Version two of the pact specification (<https://github.com/pact-foundation/pact-specification/tree/version-2>)
    V2,
    /// Version three of the pact specification (<https://github.com/pact-foundation/pact-specification/tree/version-3>)
    V3,
    /// Version four of the pact specification (<https://github.com/pact-foundation/pact-specification/tree/version-4>)
    V4,
}

impl From<NonCPactSpecification> for PactSpecification {
    #[inline]
    fn from(spec: NonCPactSpecification) -> PactSpecification {
        match spec {
            NonCPactSpecification::Unknown => PactSpecification::Unknown,
            NonCPactSpecification::V1 => PactSpecification::V1,
            NonCPactSpecification::V1_1 => PactSpecification::V1_1,
            NonCPactSpecification::V2 => PactSpecification::V2,
            NonCPactSpecification::V3 => PactSpecification::V3,
            NonCPactSpecification::V4 => PactSpecification::V4,
        }
    }
}

impl From<PactSpecification> for NonCPactSpecification {
    #[inline]
    fn from(spec: PactSpecification) -> NonCPactSpecification {
        match spec {
            PactSpecification::Unknown => NonCPactSpecification::Unknown,
            PactSpecification::V1 => NonCPactSpecification::V1,
            PactSpecification::V1_1 => NonCPactSpecification::V1_1,
            PactSpecification::V2 => NonCPactSpecification::V2,
            PactSpecification::V3 => NonCPactSpecification::V3,
            PactSpecification::V4 => NonCPactSpecification::V4,
        }
    }
}
