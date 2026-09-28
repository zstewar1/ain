use core::fmt;

/// Checks that some bytes are valid ASCII.
pub(crate) const fn run_ain_validation(bytes: &[u8]) -> Result<(), AinValidationError> {
    let mut idx = 0;
    while idx < bytes.len() {
        if !bytes[idx].is_ascii() {
            return Err(AinValidationError { valid_up_to: idx });
        }
        idx += 1;
    }
    Ok(())
}

/// Error produced from Ascii validation errors.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AinValidationError {
    /// Index of the first byte in the input that wasn't valid ASCII.
    valid_up_to: usize,
}

impl AinValidationError {
    /// Returns the index in the given string up to which valid ASCII was verified.
    ///
    /// It is the maximum index such that `AinStr::from_ascii(&input[..index])` would return
    /// `Ok(_)`.
    pub const fn valid_up_to(&self) -> usize {
        self.valid_up_to
    }
}

impl fmt::Display for AinValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid ASCII byte found at index {}", self.valid_up_to)
    }
}

impl core::error::Error for AinValidationError {}

impl From<AinValidationError> for fmt::Error {
    fn from(_: AinValidationError) -> Self {
        fmt::Error
    }
}
