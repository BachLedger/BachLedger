//! BachLedger Primitives
//!
//! Basic types for blockchain operations:
//! - `Address`: 20-byte Ethereum-compatible address
//! - `H256`: 32-byte hash value
//! - `H160`: Type alias for Address
//! - `U256`: 256-bit unsigned integer

#![forbid(unsafe_code)]

/// Length of an Ethereum-style address in bytes
pub const ADDRESS_LENGTH: usize = 20;

/// Length of a 256-bit hash in bytes
pub const HASH_LENGTH: usize = 32;

/// Errors from primitive operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveError {
    /// Slice length does not match expected size
    InvalidLength { expected: usize, actual: usize },
    /// Invalid hexadecimal character in string
    InvalidHex(String),
}

// =============================================================================
// Helper function for hex parsing
// =============================================================================

/// Parses a hex character to its 4-bit value
fn hex_char_to_nibble(c: char) -> Option<u8> {
    match c {
        '0'..='9' => Some(c as u8 - b'0'),
        'a'..='f' => Some(c as u8 - b'a' + 10),
        'A'..='F' => Some(c as u8 - b'A' + 10),
        _ => None,
    }
}

/// Parses a hex string (with or without 0x prefix) into bytes
fn parse_hex(s: &str) -> Result<Vec<u8>, PrimitiveError> {
    let s = s.strip_prefix("0x").unwrap_or(s);

    if s.is_empty() {
        return Ok(vec![]);
    }

    // Check for odd length
    if !s.len().is_multiple_of(2) {
        return Err(PrimitiveError::InvalidHex(format!(
            "hex string has odd length: {}",
            s.len()
        )));
    }

    let mut bytes = Vec::with_capacity(s.len() / 2);
    let chars: Vec<char> = s.chars().collect();

    for chunk in chars.chunks(2) {
        let high = hex_char_to_nibble(chunk[0])
            .ok_or_else(|| PrimitiveError::InvalidHex(format!("invalid hex char: {}", chunk[0])))?;
        let low = hex_char_to_nibble(chunk[1])
            .ok_or_else(|| PrimitiveError::InvalidHex(format!("invalid hex char: {}", chunk[1])))?;
        bytes.push((high << 4) | low);
    }

    Ok(bytes)
}

// =============================================================================
// Address
// =============================================================================

/// A 20-byte Ethereum-compatible address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Address([u8; ADDRESS_LENGTH]);

impl Address {
    /// Creates an Address from a byte slice.
    pub fn from_slice(slice: &[u8]) -> Result<Self, PrimitiveError> {
        if slice.len() != ADDRESS_LENGTH {
            return Err(PrimitiveError::InvalidLength {
                expected: ADDRESS_LENGTH,
                actual: slice.len(),
            });
        }
        let mut bytes = [0u8; ADDRESS_LENGTH];
        bytes.copy_from_slice(slice);
        Ok(Self(bytes))
    }

    /// Parses an Address from a hex string.
    pub fn from_hex(s: &str) -> Result<Self, PrimitiveError> {
        let bytes = parse_hex(s)?;
        if bytes.len() != ADDRESS_LENGTH {
            return Err(PrimitiveError::InvalidLength {
                expected: ADDRESS_LENGTH,
                actual: bytes.len(),
            });
        }
        let mut arr = [0u8; ADDRESS_LENGTH];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }

    /// Returns the zero address (all zeros).
    pub fn zero() -> Self {
        Self([0u8; ADDRESS_LENGTH])
    }

    /// Returns a reference to the underlying bytes.
    pub fn as_bytes(&self) -> &[u8; ADDRESS_LENGTH] {
        &self.0
    }

    /// Checks if this is the zero address.
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; ADDRESS_LENGTH]> for Address {
    fn from(bytes: [u8; ADDRESS_LENGTH]) -> Self {
        Self(bytes)
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x")?;
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl std::fmt::LowerHex for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x")?;
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

// =============================================================================
// H256
// =============================================================================

/// A 32-byte hash value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct H256([u8; HASH_LENGTH]);

impl H256 {
    /// Creates an H256 from a byte slice.
    pub fn from_slice(slice: &[u8]) -> Result<Self, PrimitiveError> {
        if slice.len() != HASH_LENGTH {
            return Err(PrimitiveError::InvalidLength {
                expected: HASH_LENGTH,
                actual: slice.len(),
            });
        }
        let mut bytes = [0u8; HASH_LENGTH];
        bytes.copy_from_slice(slice);
        Ok(Self(bytes))
    }

    /// Parses an H256 from a hex string.
    pub fn from_hex(s: &str) -> Result<Self, PrimitiveError> {
        let bytes = parse_hex(s)?;
        if bytes.len() != HASH_LENGTH {
            return Err(PrimitiveError::InvalidLength {
                expected: HASH_LENGTH,
                actual: bytes.len(),
            });
        }
        let mut arr = [0u8; HASH_LENGTH];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }

    /// Returns the zero hash (all zeros).
    pub fn zero() -> Self {
        Self([0u8; HASH_LENGTH])
    }

    /// Returns a reference to the underlying bytes.
    pub fn as_bytes(&self) -> &[u8; HASH_LENGTH] {
        &self.0
    }

    /// Checks if this is the zero hash.
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
}

impl AsRef<[u8]> for H256 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; HASH_LENGTH]> for H256 {
    fn from(bytes: [u8; HASH_LENGTH]) -> Self {
        Self(bytes)
    }
}

impl std::fmt::Display for H256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x")?;
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl std::fmt::LowerHex for H256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x")?;
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

/// Alias for Address (20-byte hash).
pub type H160 = Address;

// =============================================================================
// U256
// =============================================================================

/// A 256-bit unsigned integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct U256([u64; 4]); // Little-endian limbs: limbs[0] is the lowest 64 bits

impl U256 {
    /// Zero value.
    pub const ZERO: Self = U256([0, 0, 0, 0]);

    /// Maximum value (2^256 - 1).
    pub const MAX: Self = U256([u64::MAX, u64::MAX, u64::MAX, u64::MAX]);

    /// One value.
    pub const ONE: Self = U256([1, 0, 0, 0]);

    /// Creates a U256 from big-endian bytes.
    pub fn from_be_bytes(bytes: [u8; 32]) -> Self {
        // Big-endian: bytes[0] is most significant
        // We need to read 4 u64s in big-endian order
        // limbs[3] = bytes[0..8], limbs[2] = bytes[8..16], etc.
        let limb3 = u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        let limb2 = u64::from_be_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11],
            bytes[12], bytes[13], bytes[14], bytes[15],
        ]);
        let limb1 = u64::from_be_bytes([
            bytes[16], bytes[17], bytes[18], bytes[19],
            bytes[20], bytes[21], bytes[22], bytes[23],
        ]);
        let limb0 = u64::from_be_bytes([
            bytes[24], bytes[25], bytes[26], bytes[27],
            bytes[28], bytes[29], bytes[30], bytes[31],
        ]);
        Self([limb0, limb1, limb2, limb3])
    }

    /// Creates a U256 from little-endian bytes.
    pub fn from_le_bytes(bytes: [u8; 32]) -> Self {
        // Little-endian: bytes[0] is least significant
        let limb0 = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        let limb1 = u64::from_le_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11],
            bytes[12], bytes[13], bytes[14], bytes[15],
        ]);
        let limb2 = u64::from_le_bytes([
            bytes[16], bytes[17], bytes[18], bytes[19],
            bytes[20], bytes[21], bytes[22], bytes[23],
        ]);
        let limb3 = u64::from_le_bytes([
            bytes[24], bytes[25], bytes[26], bytes[27],
            bytes[28], bytes[29], bytes[30], bytes[31],
        ]);
        Self([limb0, limb1, limb2, limb3])
    }

    /// Converts to big-endian bytes.
    pub fn to_be_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        // limbs[3] -> bytes[0..8], limbs[2] -> bytes[8..16], etc.
        let b3 = self.0[3].to_be_bytes();
        let b2 = self.0[2].to_be_bytes();
        let b1 = self.0[1].to_be_bytes();
        let b0 = self.0[0].to_be_bytes();
        bytes[0..8].copy_from_slice(&b3);
        bytes[8..16].copy_from_slice(&b2);
        bytes[16..24].copy_from_slice(&b1);
        bytes[24..32].copy_from_slice(&b0);
        bytes
    }

    /// Converts to little-endian bytes.
    pub fn to_le_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        let b0 = self.0[0].to_le_bytes();
        let b1 = self.0[1].to_le_bytes();
        let b2 = self.0[2].to_le_bytes();
        let b3 = self.0[3].to_le_bytes();
        bytes[0..8].copy_from_slice(&b0);
        bytes[8..16].copy_from_slice(&b1);
        bytes[16..24].copy_from_slice(&b2);
        bytes[24..32].copy_from_slice(&b3);
        bytes
    }

    /// Creates from a u64 value.
    pub fn from_u64(val: u64) -> Self {
        Self([val, 0, 0, 0])
    }

    /// Checked addition. Returns None on overflow.
    pub fn checked_add(&self, other: &Self) -> Option<Self> {
        let mut result = [0u64; 4];
        let mut carry = 0u64;

        for i in 0..4 {
            let (sum1, c1) = self.0[i].overflowing_add(other.0[i]);
            let (sum2, c2) = sum1.overflowing_add(carry);
            result[i] = sum2;
            carry = (c1 as u64) + (c2 as u64);
        }

        if carry != 0 {
            None
        } else {
            Some(Self(result))
        }
    }

    /// Checked subtraction. Returns None on underflow.
    pub fn checked_sub(&self, other: &Self) -> Option<Self> {
        let mut result = [0u64; 4];
        let mut borrow = 0u64;

        for i in 0..4 {
            let (diff1, b1) = self.0[i].overflowing_sub(other.0[i]);
            let (diff2, b2) = diff1.overflowing_sub(borrow);
            result[i] = diff2;
            borrow = (b1 as u64) + (b2 as u64);
        }

        if borrow != 0 {
            None
        } else {
            Some(Self(result))
        }
    }

    /// Checked multiplication. Returns None on overflow.
    pub fn checked_mul(&self, other: &Self) -> Option<Self> {
        // Handle zero cases early
        if self.is_zero() || other.is_zero() {
            return Some(Self::ZERO);
        }

        // Use schoolbook multiplication with 64-bit limbs
        // Track result in 8 u64 limbs to detect overflow
        let mut result = [0u64; 8];

        for i in 0..4 {
            if self.0[i] == 0 {
                continue;
            }
            let mut carry: u64 = 0;
            for j in 0..4 {
                let idx = i + j;
                // product = self.0[i] * other.0[j] + result[idx] + carry
                // This can be up to: (2^64-1)*(2^64-1) + (2^64-1) + (2^64-1)
                //                  = 2^128 - 2^65 + 1 + 2^65 - 2 = 2^128 - 1
                // So it fits in u128
                let product = (self.0[i] as u128) * (other.0[j] as u128)
                            + (result[idx] as u128)
                            + (carry as u128);
                result[idx] = product as u64;
                carry = (product >> 64) as u64;
            }
            // Propagate remaining carry
            let mut k = i + 4;
            while carry != 0 && k < 8 {
                let sum = (result[k] as u128) + (carry as u128);
                result[k] = sum as u64;
                carry = (sum >> 64) as u64;
                k += 1;
            }
            if carry != 0 {
                return None; // Overflow beyond 512 bits (shouldn't happen for 256x256)
            }
        }

        // Check if result fits in 256 bits (upper 4 limbs must be zero)
        if result[4] != 0 || result[5] != 0 || result[6] != 0 || result[7] != 0 {
            return None;
        }

        Some(Self([result[0], result[1], result[2], result[3]]))
    }

    /// Checked division. Returns None if divisor is zero.
    pub fn checked_div(&self, other: &Self) -> Option<Self> {
        if other.is_zero() {
            return None;
        }

        if self.is_zero() {
            return Some(Self::ZERO);
        }

        // If self < other, result is 0
        if self < other {
            return Some(Self::ZERO);
        }

        // If self == other, result is 1
        if self == other {
            return Some(Self::ONE);
        }

        // Binary long division
        let mut quotient = Self::ZERO;
        let mut remainder = *self;

        // Find the highest bit set in divisor
        let divisor_bits = other.bits();

        // Find the highest bit set in dividend
        let dividend_bits = self.bits();

        if divisor_bits == 0 {
            return None; // divisor is zero
        }

        // Shift amount to align divisor with dividend
        let mut shift = dividend_bits.saturating_sub(divisor_bits);

        // Shifted divisor
        let mut shifted_divisor = other.shl(shift);

        loop {
            if remainder >= shifted_divisor {
                remainder = remainder.checked_sub(&shifted_divisor).unwrap();
                quotient = quotient.set_bit(shift);
            }

            if shift == 0 {
                break;
            }

            shift -= 1;
            shifted_divisor = shifted_divisor.shr1();
        }

        Some(quotient)
    }

    /// Returns true if value is zero.
    pub fn is_zero(&self) -> bool {
        self.0[0] == 0 && self.0[1] == 0 && self.0[2] == 0 && self.0[3] == 0
    }

    /// Wrapping addition (modulo 2^256)
    pub fn wrapping_add(&self, other: &Self) -> Self {
        let mut result = [0u64; 4];
        let mut carry = 0u64;
        for i in 0..4 {
            let (sum1, c1) = self.0[i].overflowing_add(other.0[i]);
            let (sum2, c2) = sum1.overflowing_add(carry);
            result[i] = sum2;
            carry = (c1 as u64) + (c2 as u64);
        }
        Self(result)
    }

    /// Wrapping subtraction (modulo 2^256)
    pub fn wrapping_sub(&self, other: &Self) -> Self {
        let mut result = [0u64; 4];
        let mut borrow = 0u64;
        for i in 0..4 {
            let (diff1, b1) = self.0[i].overflowing_sub(other.0[i]);
            let (diff2, b2) = diff1.overflowing_sub(borrow);
            result[i] = diff2;
            borrow = (b1 as u64) + (b2 as u64);
        }
        Self(result)
    }

    /// Wrapping multiplication (modulo 2^256)
    pub fn wrapping_mul(&self, other: &Self) -> Self {
        if self.is_zero() || other.is_zero() {
            return Self::ZERO;
        }
        let mut result = [0u64; 4];
        for i in 0..4 {
            if self.0[i] == 0 {
                continue;
            }
            let mut carry: u64 = 0;
            for j in 0..4 {
                if i + j >= 4 {
                    break;
                }
                let product = (self.0[i] as u128) * (other.0[j] as u128)
                    + (result[i + j] as u128)
                    + (carry as u128);
                result[i + j] = product as u64;
                carry = (product >> 64) as u64;
            }
        }
        Self(result)
    }

    /// Wrapping modulo
    pub fn wrapping_mod(&self, other: &Self) -> Self {
        if other.is_zero() {
            return Self::ZERO;
        }
        let (_, rem) = self.div_rem(other);
        rem
    }

    /// Division with remainder
    pub fn div_rem(&self, other: &Self) -> (Self, Self) {
        if other.is_zero() {
            return (Self::ZERO, Self::ZERO);
        }
        if self.is_zero() {
            return (Self::ZERO, Self::ZERO);
        }
        if self < other {
            return (Self::ZERO, *self);
        }
        if self == other {
            return (Self::ONE, Self::ZERO);
        }

        let mut quotient = Self::ZERO;
        let mut remainder = *self;
        let divisor_bits = other.bits();
        let dividend_bits = self.bits();
        let mut shift = dividend_bits.saturating_sub(divisor_bits);
        let mut shifted_divisor = other.shl(shift);

        loop {
            if remainder >= shifted_divisor {
                remainder = remainder.checked_sub(&shifted_divisor).unwrap();
                quotient = quotient.set_bit(shift);
            }
            if shift == 0 {
                break;
            }
            shift -= 1;
            shifted_divisor = shifted_divisor.shr1();
        }
        (quotient, remainder)
    }

    /// Bitwise AND
    pub fn bitand(&self, other: &Self) -> Self {
        Self([
            self.0[0] & other.0[0],
            self.0[1] & other.0[1],
            self.0[2] & other.0[2],
            self.0[3] & other.0[3],
        ])
    }

    /// Bitwise OR
    pub fn bitor(&self, other: &Self) -> Self {
        Self([
            self.0[0] | other.0[0],
            self.0[1] | other.0[1],
            self.0[2] | other.0[2],
            self.0[3] | other.0[3],
        ])
    }

    /// Bitwise XOR
    pub fn bitxor(&self, other: &Self) -> Self {
        Self([
            self.0[0] ^ other.0[0],
            self.0[1] ^ other.0[1],
            self.0[2] ^ other.0[2],
            self.0[3] ^ other.0[3],
        ])
    }

    /// Bitwise NOT
    pub fn bitnot(&self) -> Self {
        Self([!self.0[0], !self.0[1], !self.0[2], !self.0[3]])
    }

    /// Convert to usize (truncates)
    pub fn as_usize(&self) -> usize {
        self.0[0] as usize
    }

    /// Convert to u64 (truncates)
    pub fn as_u64(&self) -> u64 {
        self.0[0]
    }

    /// Check if high bit is set (negative in two's complement)
    pub fn is_negative(&self) -> bool {
        self.0[3] & (1 << 63) != 0
    }

    /// Two's complement negation
    pub fn twos_complement(&self) -> Self {
        self.bitnot().wrapping_add(&Self::ONE)
    }

    /// Left shift by n bits
    pub fn shl(&self, n: usize) -> Self {
        if n == 0 {
            return *self;
        }
        if n >= 256 {
            return Self::ZERO;
        }
        let limb_shift = n / 64;
        let bit_shift = n % 64;
        let mut result = [0u64; 4];
        if bit_shift == 0 {
            for i in limb_shift..4 {
                result[i] = self.0[i - limb_shift];
            }
        } else {
            for i in limb_shift..4 {
                result[i] = self.0[i - limb_shift] << bit_shift;
                if i > limb_shift {
                    result[i] |= self.0[i - limb_shift - 1] >> (64 - bit_shift);
                }
            }
        }
        Self(result)
    }

    /// Right shift by n bits
    pub fn shr(&self, n: usize) -> Self {
        if n == 0 {
            return *self;
        }
        if n >= 256 {
            return Self::ZERO;
        }
        let limb_shift = n / 64;
        let bit_shift = n % 64;
        let mut result = [0u64; 4];
        if bit_shift == 0 {
            for i in 0..(4 - limb_shift) {
                result[i] = self.0[i + limb_shift];
            }
        } else {
            for i in 0..(4 - limb_shift) {
                result[i] = self.0[i + limb_shift] >> bit_shift;
                if i + limb_shift + 1 < 4 {
                    result[i] |= self.0[i + limb_shift + 1] << (64 - bit_shift);
                }
            }
        }
        Self(result)
    }

    /// Get the internal limbs (for EVM operations)
    pub fn limbs(&self) -> &[u64; 4] {
        &self.0
    }

    /// Create from internal limbs
    pub fn from_limbs(limbs: [u64; 4]) -> Self {
        Self(limbs)
    }

    /// Returns the number of bits needed to represent this value
    fn bits(&self) -> usize {
        for i in (0..4).rev() {
            if self.0[i] != 0 {
                return (i + 1) * 64 - self.0[i].leading_zeros() as usize;
            }
        }
        0
    }

    /// Right shift by 1 bit (internal helper)
    fn shr1(&self) -> Self {
        let mut result = [0u64; 4];

        for i in 0..4 {
            result[i] = self.0[i] >> 1;
            if i < 3 {
                result[i] |= self.0[i + 1] << 63;
            }
        }

        Self(result)
    }

    /// Set bit at position n (internal helper)
    fn set_bit(&self, n: usize) -> Self {
        if n >= 256 {
            return *self;
        }
        let limb_idx = n / 64;
        let bit_idx = n % 64;
        let mut result = *self;
        result.0[limb_idx] |= 1u64 << bit_idx;
        result
    }
}

impl From<u64> for U256 {
    fn from(val: u64) -> Self {
        Self::from_u64(val)
    }
}

impl From<u128> for U256 {
    fn from(val: u128) -> Self {
        Self([val as u64, (val >> 64) as u64, 0, 0])
    }
}

impl PartialOrd for U256 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for U256 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Compare from most significant limb to least significant
        for i in (0..4).rev() {
            match self.0[i].cmp(&other.0[i]) {
                std::cmp::Ordering::Equal => continue,
                other_ord => return other_ord,
            }
        }
        std::cmp::Ordering::Equal
    }
}

impl std::fmt::Display for U256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_zero() {
            return write!(f, "0");
        }

        // Convert to decimal by repeated division by 10
        let mut digits = Vec::new();
        let mut val = *self;
        let ten = U256::from_u64(10);

        while !val.is_zero() {
            let (quot, rem) = val.div_rem(&ten);
            digits.push((rem.0[0] as u8) + b'0');
            val = quot;
        }

        digits.reverse();
        let s: String = digits.iter().map(|&b| b as char).collect();
        write!(f, "{}", s)
    }
}

impl std::fmt::LowerHex for U256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_zero() {
            return write!(f, "0x0");
        }

        // Find the first non-zero limb from the most significant
        let mut started = false;
        write!(f, "0x")?;

        for i in (0..4).rev() {
            if !started && self.0[i] == 0 {
                continue;
            }

            if !started {
                // First non-zero limb: no leading zeros
                write!(f, "{:x}", self.0[i])?;
                started = true;
            } else {
                // Subsequent limbs: pad with zeros to 16 hex chars
                write!(f, "{:016x}", self.0[i])?;
            }
        }

        Ok(())
    }
}
