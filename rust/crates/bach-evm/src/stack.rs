//! EVM stack implementation

use crate::error::{EvmError, EvmResult};
use crate::gas::cost::MAX_STACK_SIZE;

/// 256-bit unsigned integer for EVM stack (big-endian [u8; 32])
pub type U256 = [u8; 32];

/// Internal alias for primitive_types::U256 used for arithmetic
type PU256 = primitive_types::U256;

/// Zero value
pub const U256_ZERO: U256 = [0u8; 32];

/// One value
pub const U256_ONE: U256 = {
    let mut v = [0u8; 32];
    v[31] = 1;
    v
};

/// Max value
pub const U256_MAX: U256 = [0xFF; 32];

/// EVM stack (max 1024 items, 256-bit each)
#[derive(Clone, Debug)]
pub struct Stack {
    data: Vec<U256>,
}

impl Stack {
    /// Create a new empty stack
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(MAX_STACK_SIZE),
        }
    }

    /// Push a value onto the stack
    pub fn push(&mut self, value: U256) -> EvmResult<()> {
        if self.data.len() >= MAX_STACK_SIZE {
            return Err(EvmError::StackOverflow);
        }
        self.data.push(value);
        Ok(())
    }

    /// Pop a value from the stack
    pub fn pop(&mut self) -> EvmResult<U256> {
        self.data.pop().ok_or(EvmError::StackUnderflow)
    }

    /// Peek at the top of the stack
    pub fn peek(&self) -> EvmResult<&U256> {
        self.data.last().ok_or(EvmError::StackUnderflow)
    }

    /// Peek at a specific depth (0 = top)
    pub fn peek_at(&self, depth: usize) -> EvmResult<&U256> {
        if depth >= self.data.len() {
            return Err(EvmError::StackUnderflow);
        }
        Ok(&self.data[self.data.len() - 1 - depth])
    }

    /// Swap top with item at depth (1 = swap with second item)
    pub fn swap(&mut self, depth: usize) -> EvmResult<()> {
        if depth == 0 || depth > self.data.len() - 1 {
            return Err(EvmError::StackUnderflow);
        }
        let len = self.data.len();
        self.data.swap(len - 1, len - 1 - depth);
        Ok(())
    }

    /// Duplicate item at depth to top (1 = dup top)
    pub fn dup(&mut self, depth: usize) -> EvmResult<()> {
        if depth == 0 || depth > self.data.len() {
            return Err(EvmError::StackUnderflow);
        }
        if self.data.len() >= MAX_STACK_SIZE {
            return Err(EvmError::StackOverflow);
        }
        let value = self.data[self.data.len() - depth];
        self.data.push(value);
        Ok(())
    }

    /// Get current stack size
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if stack is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clear the stack
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

// U256 helper functions

/// Convert u64 to U256
pub fn u64_to_u256(value: u64) -> U256 {
    let mut result = U256_ZERO;
    result[24..32].copy_from_slice(&value.to_be_bytes());
    result
}

/// Convert u128 to U256
pub fn u128_to_u256(value: u128) -> U256 {
    let mut result = U256_ZERO;
    result[16..32].copy_from_slice(&value.to_be_bytes());
    result
}

/// Try to convert U256 to u64 (returns None if overflow)
pub fn u256_to_u64(value: &U256) -> Option<u64> {
    // Check high bytes are zero
    if value[0..24].iter().any(|&b| b != 0) {
        return None;
    }
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&value[24..32]);
    Some(u64::from_be_bytes(bytes))
}

/// Try to convert U256 to usize (returns None if overflow)
pub fn u256_to_usize(value: &U256) -> Option<usize> {
    u256_to_u64(value).and_then(|v| usize::try_from(v).ok())
}

/// Check if U256 is zero
pub fn u256_is_zero(value: &U256) -> bool {
    value.iter().all(|&b| b == 0)
}

/// U256 addition with overflow check
pub fn u256_add(a: &U256, b: &U256) -> U256 {
    let mut result = U256_ZERO;
    let mut carry = 0u16;

    for i in (0..32).rev() {
        let sum = a[i] as u16 + b[i] as u16 + carry;
        result[i] = sum as u8;
        carry = sum >> 8;
    }

    result
}

/// U256 subtraction with underflow wrapping
pub fn u256_sub(a: &U256, b: &U256) -> U256 {
    let mut result = U256_ZERO;
    let mut borrow = 0i16;

    for i in (0..32).rev() {
        let diff = a[i] as i16 - b[i] as i16 - borrow;
        if diff < 0 {
            result[i] = (diff + 256) as u8;
            borrow = 1;
        } else {
            result[i] = diff as u8;
            borrow = 0;
        }
    }

    result
}

/// U256 comparison (returns -1, 0, 1)
pub fn u256_cmp(a: &U256, b: &U256) -> std::cmp::Ordering {
    for i in 0..32 {
        match a[i].cmp(&b[i]) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    std::cmp::Ordering::Equal
}

/// U256 less than
pub fn u256_lt(a: &U256, b: &U256) -> bool {
    u256_cmp(a, b) == std::cmp::Ordering::Less
}

/// U256 greater than
pub fn u256_gt(a: &U256, b: &U256) -> bool {
    u256_cmp(a, b) == std::cmp::Ordering::Greater
}

/// U256 bitwise AND
pub fn u256_and(a: &U256, b: &U256) -> U256 {
    let mut result = U256_ZERO;
    for i in 0..32 {
        result[i] = a[i] & b[i];
    }
    result
}

/// U256 bitwise OR
pub fn u256_or(a: &U256, b: &U256) -> U256 {
    let mut result = U256_ZERO;
    for i in 0..32 {
        result[i] = a[i] | b[i];
    }
    result
}

/// U256 bitwise XOR
pub fn u256_xor(a: &U256, b: &U256) -> U256 {
    let mut result = U256_ZERO;
    for i in 0..32 {
        result[i] = a[i] ^ b[i];
    }
    result
}

/// U256 bitwise NOT
pub fn u256_not(a: &U256) -> U256 {
    let mut result = U256_ZERO;
    for i in 0..32 {
        result[i] = !a[i];
    }
    result
}

// ==================== Conversion helpers ====================

/// Convert [u8; 32] big-endian to primitive_types::U256
fn to_pu256(v: &U256) -> PU256 {
    PU256::from_big_endian(v)
}

/// Convert primitive_types::U256 to [u8; 32] big-endian
fn from_pu256(v: PU256) -> U256 {
    let mut result = U256_ZERO;
    v.to_big_endian(&mut result);
    result
}

// ==================== Full 256-bit arithmetic ====================

/// U256 multiplication (full 256-bit, wrapping)
pub fn u256_mul(a: &U256, b: &U256) -> U256 {
    let pa = to_pu256(a);
    let pb = to_pu256(b);
    from_pu256(pa.overflowing_mul(pb).0)
}

/// U256 division (full 256-bit, returns 0 on div by zero)
pub fn u256_div(a: &U256, b: &U256) -> U256 {
    let pb = to_pu256(b);
    if pb.is_zero() {
        return U256_ZERO;
    }
    from_pu256(to_pu256(a) / pb)
}

/// U256 modulo (full 256-bit, returns 0 on mod by zero)
pub fn u256_mod(a: &U256, b: &U256) -> U256 {
    let pb = to_pu256(b);
    if pb.is_zero() {
        return U256_ZERO;
    }
    from_pu256(to_pu256(a) % pb)
}

/// U256 signed division (two's complement)
pub fn u256_sdiv(a: &U256, b: &U256) -> U256 {
    let pb = to_pu256(b);
    if pb.is_zero() {
        return U256_ZERO;
    }
    let pa = to_pu256(a);
    let a_neg = is_negative(&pa);
    let b_neg = is_negative(&pb);
    let abs_a = if a_neg { twos_complement(pa) } else { pa };
    let abs_b = if b_neg { twos_complement(pb) } else { pb };
    let result = abs_a / abs_b;
    if a_neg != b_neg {
        from_pu256(twos_complement(result))
    } else {
        from_pu256(result)
    }
}

/// U256 signed modulo (two's complement, sign of dividend)
pub fn u256_smod(a: &U256, b: &U256) -> U256 {
    let pb = to_pu256(b);
    if pb.is_zero() {
        return U256_ZERO;
    }
    let pa = to_pu256(a);
    let a_neg = is_negative(&pa);
    let b_neg = is_negative(&pb);
    let abs_a = if a_neg { twos_complement(pa) } else { pa };
    let abs_b = if b_neg { twos_complement(pb) } else { pb };
    let result = abs_a % abs_b;
    if a_neg {
        from_pu256(twos_complement(result))
    } else {
        from_pu256(result)
    }
}

/// (a + b) % N, with no overflow on intermediate sum (uses 512-bit internally)
pub fn u256_addmod(a: &U256, b: &U256, n: &U256) -> U256 {
    let pn = to_pu256(n);
    if pn.is_zero() {
        return U256_ZERO;
    }
    let pa = to_pu256(a);
    let pb = to_pu256(b);
    // Use primitive_types overflow detection
    let (sum, overflow) = pa.overflowing_add(pb);
    if overflow {
        // sum = (a + b) mod 2^256, actual = sum + 2^256
        // (a + b) mod n = ((sum mod n) + (2^256 mod n)) mod n
        let max_plus_one_mod_n = (PU256::MAX % pn).overflowing_add(PU256::one()).0 % pn;
        from_pu256((sum % pn + max_plus_one_mod_n) % pn)
    } else {
        from_pu256(sum % pn)
    }
}

/// (a * b) % N, with no overflow on intermediate product
pub fn u256_mulmod(a: &U256, b: &U256, n: &U256) -> U256 {
    let pn = to_pu256(n);
    if pn.is_zero() {
        return U256_ZERO;
    }
    let pa = to_pu256(a);
    let pb = to_pu256(b);
    // For full correctness, we need 512-bit intermediate.
    // Use schoolbook: split into halves and combine.
    from_pu256(mulmod_512(pa, pb, pn))
}

/// 512-bit mulmod implementation
fn mulmod_512(a: PU256, b: PU256, n: PU256) -> PU256 {
    if n <= PU256::one() {
        return PU256::zero();
    }
    // Russian peasant multiplication with modular reduction
    let mut result = PU256::zero();
    let mut a = a % n;
    let mut b = b;
    while !b.is_zero() {
        if b & PU256::one() != PU256::zero() {
            result = {
                let (s, overflow) = result.overflowing_add(a);
                if overflow || s >= n { s.overflowing_sub(n).0 } else { s }
            };
        }
        a = {
            let (s, overflow) = a.overflowing_add(a);
            if overflow || s >= n { s.overflowing_sub(n).0 } else { s }
        };
        b = b >> 1;
    }
    result
}

/// U256 exponentiation (modular: base^exp mod 2^256)
pub fn u256_exp(base: &U256, exp: &U256) -> U256 {
    let mut b = to_pu256(base);
    let mut e = to_pu256(exp);
    let mut result = PU256::one();
    while !e.is_zero() {
        if e & PU256::one() != PU256::zero() {
            result = result.overflowing_mul(b).0;
        }
        b = b.overflowing_mul(b).0;
        e = e >> 1;
    }
    from_pu256(result)
}

/// SIGNEXTEND: sign-extend byte b from value x
pub fn u256_signextend(b: &U256, x: &U256) -> U256 {
    let pb = to_pu256(b);
    if pb >= PU256::from(31) {
        // No extension needed if b >= 31
        return *x;
    }
    let bit_index = (pb.low_u64() as usize) * 8 + 7;
    let byte_index = 31 - (pb.low_u64() as usize);
    let sign_bit = (x[byte_index] >> 7) & 1;
    let mut result = *x;
    if sign_bit == 1 {
        // Set all higher bits to 1
        for i in 0..byte_index {
            result[i] = 0xFF;
        }
        result[byte_index] |= !((1u8 << (bit_index % 8 + 1)).wrapping_sub(1));
    } else {
        // Set all higher bits to 0
        for i in 0..byte_index {
            result[i] = 0x00;
        }
        result[byte_index] &= (1u8 << (bit_index % 8 + 1)).wrapping_sub(1);
    }
    result
}

/// BYTE: extract byte at position i from value (0 = most significant)
pub fn u256_byte(i: &U256, x: &U256) -> U256 {
    let pi = to_pu256(i);
    if pi >= PU256::from(32) {
        return U256_ZERO;
    }
    let idx = pi.low_u64() as usize;
    let mut result = U256_ZERO;
    result[31] = x[idx];
    result
}

/// SHL: shift left (shift, value) -> value << shift
pub fn u256_shl(shift: &U256, value: &U256) -> U256 {
    let ps = to_pu256(shift);
    if ps >= PU256::from(256) {
        return U256_ZERO;
    }
    from_pu256(to_pu256(value) << ps.as_u32())
}

/// SHR: logical shift right (shift, value) -> value >> shift
pub fn u256_shr(shift: &U256, value: &U256) -> U256 {
    let ps = to_pu256(shift);
    if ps >= PU256::from(256) {
        return U256_ZERO;
    }
    from_pu256(to_pu256(value) >> ps.as_u32())
}

/// SAR: arithmetic shift right (shift, value) -> value >>> shift (sign-preserving)
pub fn u256_sar(shift: &U256, value: &U256) -> U256 {
    let ps = to_pu256(shift);
    let pv = to_pu256(value);
    let negative = is_negative(&pv);

    if ps >= PU256::from(256) {
        return if negative { U256_MAX } else { U256_ZERO };
    }

    let shifted = pv >> ps.as_u32();
    if negative {
        // Fill high bits with 1s
        let mask = PU256::MAX << (PU256::from(256) - ps);
        from_pu256(shifted | mask)
    } else {
        from_pu256(shifted)
    }
}

/// Signed less than comparison (two's complement)
pub fn u256_slt(a: &U256, b: &U256) -> bool {
    let pa = to_pu256(a);
    let pb = to_pu256(b);
    let a_neg = is_negative(&pa);
    let b_neg = is_negative(&pb);
    match (a_neg, b_neg) {
        (true, false) => true,   // negative < positive
        (false, true) => false,  // positive > negative
        _ => pa < pb,            // same sign: compare unsigned
    }
}

/// Signed greater than comparison (two's complement)
pub fn u256_sgt(a: &U256, b: &U256) -> bool {
    u256_slt(b, a)
}

// ==================== Internal helpers ====================

/// Check if a value is negative in two's complement
fn is_negative(v: &PU256) -> bool {
    v.bit(255)
}

/// Two's complement negation: ~v + 1
fn twos_complement(v: PU256) -> PU256 {
    (!v).overflowing_add(PU256::one()).0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_push_pop() {
        let mut stack = Stack::new();

        stack.push(u64_to_u256(42)).unwrap();
        stack.push(u64_to_u256(100)).unwrap();

        assert_eq!(stack.len(), 2);
        assert_eq!(u256_to_u64(&stack.pop().unwrap()), Some(100));
        assert_eq!(u256_to_u64(&stack.pop().unwrap()), Some(42));
        assert!(stack.is_empty());
    }

    #[test]
    fn test_stack_underflow() {
        let mut stack = Stack::new();
        assert!(matches!(stack.pop(), Err(EvmError::StackUnderflow)));
    }

    #[test]
    fn test_stack_overflow() {
        let mut stack = Stack::new();
        for i in 0..1024 {
            stack.push(u64_to_u256(i)).unwrap();
        }
        assert!(matches!(stack.push(U256_ZERO), Err(EvmError::StackOverflow)));
    }

    #[test]
    fn test_stack_dup() {
        let mut stack = Stack::new();
        stack.push(u64_to_u256(1)).unwrap();
        stack.push(u64_to_u256(2)).unwrap();
        stack.push(u64_to_u256(3)).unwrap();

        stack.dup(2).unwrap(); // Dup second from top (value 2)
        assert_eq!(u256_to_u64(&stack.pop().unwrap()), Some(2));
    }

    #[test]
    fn test_stack_swap() {
        let mut stack = Stack::new();
        stack.push(u64_to_u256(1)).unwrap();
        stack.push(u64_to_u256(2)).unwrap();
        stack.push(u64_to_u256(3)).unwrap();

        stack.swap(2).unwrap(); // Swap top with third
        assert_eq!(u256_to_u64(&stack.pop().unwrap()), Some(1));
        assert_eq!(u256_to_u64(&stack.pop().unwrap()), Some(2));
        assert_eq!(u256_to_u64(&stack.pop().unwrap()), Some(3));
    }

    #[test]
    fn test_u256_arithmetic() {
        let a = u64_to_u256(100);
        let b = u64_to_u256(50);

        let sum = u256_add(&a, &b);
        assert_eq!(u256_to_u64(&sum), Some(150));

        let diff = u256_sub(&a, &b);
        assert_eq!(u256_to_u64(&diff), Some(50));
    }

    #[test]
    fn test_u256_comparison() {
        let a = u64_to_u256(100);
        let b = u64_to_u256(50);

        assert!(u256_gt(&a, &b));
        assert!(u256_lt(&b, &a));
        assert!(!u256_lt(&a, &a));
    }

    #[test]
    fn test_u256_bitwise() {
        let a = u64_to_u256(0xFF);
        let b = u64_to_u256(0x0F);

        let and_result = u256_and(&a, &b);
        assert_eq!(u256_to_u64(&and_result), Some(0x0F));

        let or_result = u256_or(&a, &b);
        assert_eq!(u256_to_u64(&or_result), Some(0xFF));

        let xor_result = u256_xor(&a, &b);
        assert_eq!(u256_to_u64(&xor_result), Some(0xF0));
    }

    // ==================== Extended Stack Tests ====================

    #[test]
    fn test_stack_peek() {
        let mut stack = Stack::new();
        assert!(stack.peek().is_err());

        stack.push(u64_to_u256(42)).unwrap();
        assert_eq!(u256_to_u64(stack.peek().unwrap()), Some(42));
        assert_eq!(stack.len(), 1); // Peek doesn't remove
    }

    #[test]
    fn test_stack_peek_at() {
        let mut stack = Stack::new();
        stack.push(u64_to_u256(1)).unwrap();
        stack.push(u64_to_u256(2)).unwrap();
        stack.push(u64_to_u256(3)).unwrap();

        assert_eq!(u256_to_u64(stack.peek_at(0).unwrap()), Some(3)); // Top
        assert_eq!(u256_to_u64(stack.peek_at(1).unwrap()), Some(2));
        assert_eq!(u256_to_u64(stack.peek_at(2).unwrap()), Some(1)); // Bottom
        assert!(stack.peek_at(3).is_err()); // Beyond stack
    }

    #[test]
    fn test_stack_clear() {
        let mut stack = Stack::new();
        stack.push(u64_to_u256(1)).unwrap();
        stack.push(u64_to_u256(2)).unwrap();
        assert_eq!(stack.len(), 2);

        stack.clear();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn test_stack_default() {
        let stack: Stack = Default::default();
        assert!(stack.is_empty());
    }

    #[test]
    fn test_stack_dup_overflow() {
        let mut stack = Stack::new();
        // Fill stack to max - 1
        for i in 0..1023 {
            stack.push(u64_to_u256(i)).unwrap();
        }
        // Now dup should succeed
        stack.dup(1).unwrap();
        // Stack is now full, another dup should fail
        assert!(matches!(stack.dup(1), Err(EvmError::StackOverflow)));
    }

    #[test]
    fn test_stack_dup_underflow() {
        let mut stack = Stack::new();
        stack.push(u64_to_u256(1)).unwrap();
        // DUP1 should work (dup top)
        stack.dup(1).unwrap();
        // DUP3 should fail (only 2 elements)
        assert!(matches!(stack.dup(3), Err(EvmError::StackUnderflow)));
    }

    #[test]
    fn test_stack_dup_zero_invalid() {
        let mut stack = Stack::new();
        stack.push(u64_to_u256(1)).unwrap();
        // DUP0 is invalid
        assert!(matches!(stack.dup(0), Err(EvmError::StackUnderflow)));
    }

    #[test]
    fn test_stack_swap_zero_invalid() {
        let mut stack = Stack::new();
        stack.push(u64_to_u256(1)).unwrap();
        stack.push(u64_to_u256(2)).unwrap();
        // SWAP0 is invalid
        assert!(matches!(stack.swap(0), Err(EvmError::StackUnderflow)));
    }

    #[test]
    fn test_stack_swap_underflow() {
        let mut stack = Stack::new();
        stack.push(u64_to_u256(1)).unwrap();
        // SWAP1 needs at least 2 elements
        assert!(matches!(stack.swap(1), Err(EvmError::StackUnderflow)));
    }

    #[test]
    fn test_stack_all_dup_operations() {
        // Test DUP1 through DUP16
        for depth in 1..=16 {
            let mut stack = Stack::new();
            for i in 0..depth {
                stack.push(u64_to_u256(i as u64)).unwrap();
            }
            stack.dup(depth).unwrap();
            // The duplicated value should be the bottom element (0)
            assert_eq!(u256_to_u64(&stack.pop().unwrap()), Some(0));
        }
    }

    #[test]
    fn test_stack_all_swap_operations() {
        // Test SWAP1 through SWAP16
        for depth in 1..=16 {
            let mut stack = Stack::new();
            // Push depth+1 elements
            for i in 0..=depth {
                stack.push(u64_to_u256(i as u64)).unwrap();
            }
            // Top should be `depth`, bottom should be 0
            stack.swap(depth).unwrap();
            // After swap, top should be 0 and position `depth` should be `depth`
            assert_eq!(u256_to_u64(&stack.pop().unwrap()), Some(0));
        }
    }

    // ==================== Extended U256 Tests ====================

    #[test]
    fn test_u256_constants() {
        assert!(u256_is_zero(&U256_ZERO));
        assert!(!u256_is_zero(&U256_ONE));
        assert!(!u256_is_zero(&U256_MAX));

        assert_eq!(u256_to_u64(&U256_ZERO), Some(0));
        assert_eq!(u256_to_u64(&U256_ONE), Some(1));
        assert_eq!(u256_to_u64(&U256_MAX), None); // Overflow
    }

    #[test]
    fn test_u256_add_overflow() {
        // MAX + 1 should wrap to 0
        let result = u256_add(&U256_MAX, &U256_ONE);
        assert!(u256_is_zero(&result));

        // MAX + MAX should wrap to MAX - 1
        let result = u256_add(&U256_MAX, &U256_MAX);
        let expected = u256_sub(&U256_MAX, &U256_ONE);
        // Check wrap: MAX + MAX = 2*MAX = 2^256 - 2 = (2^256 - 1) - 1 = MAX - 1
        // But with overflow: result = (2*MAX) mod 2^256 = -2 mod 2^256 = MAX - 1
        assert_eq!(result, expected);
    }

    #[test]
    fn test_u256_sub_underflow() {
        // 0 - 1 should wrap to MAX
        let result = u256_sub(&U256_ZERO, &U256_ONE);
        assert_eq!(result, U256_MAX);

        // 1 - 2 should wrap to MAX
        let one = u64_to_u256(1);
        let two = u64_to_u256(2);
        let result = u256_sub(&one, &two);
        assert_eq!(result, U256_MAX);
    }

    #[test]
    fn test_u256_comparison_equal() {
        let a = u64_to_u256(100);
        let b = u64_to_u256(100);
        assert!(!u256_lt(&a, &b));
        assert!(!u256_gt(&a, &b));
        assert_eq!(u256_cmp(&a, &b), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_u256_comparison_max() {
        // MAX should be greater than any smaller value
        let small = u64_to_u256(1000);
        assert!(u256_gt(&U256_MAX, &small));
        assert!(u256_lt(&small, &U256_MAX));
    }

    #[test]
    fn test_u256_not() {
        // NOT 0 = MAX
        let result = u256_not(&U256_ZERO);
        assert_eq!(result, U256_MAX);

        // NOT MAX = 0
        let result = u256_not(&U256_MAX);
        assert_eq!(result, U256_ZERO);

        // NOT NOT x = x
        let x = u64_to_u256(0x12345678);
        let result = u256_not(&u256_not(&x));
        assert_eq!(result, x);
    }

    #[test]
    fn test_u128_to_u256() {
        let val: u128 = 0xFFFFFFFFFFFFFFFF_FFFFFFFFFFFFFFFF;
        let u256_val = u128_to_u256(val);
        // The value should be in the lower 16 bytes
        assert!(u256_val[..16].iter().all(|&b| b == 0));
        assert!(u256_val[16..].iter().all(|&b| b == 0xFF));
    }

    #[test]
    fn test_u256_to_usize() {
        let small = u64_to_u256(1000);
        assert_eq!(u256_to_usize(&small), Some(1000));

        // Large value that fits in u64 but maybe not usize (depends on platform)
        let big = u64_to_u256(u64::MAX);
        // On 64-bit platforms, this should work
        #[cfg(target_pointer_width = "64")]
        assert_eq!(u256_to_usize(&big), Some(u64::MAX as usize));
    }

    #[test]
    fn test_u256_is_zero_edge_cases() {
        // Almost zero (only last byte is 1)
        let mut almost_zero = U256_ZERO;
        almost_zero[31] = 1;
        assert!(!u256_is_zero(&almost_zero));

        // First byte is 1
        let mut first_byte = U256_ZERO;
        first_byte[0] = 1;
        assert!(!u256_is_zero(&first_byte));
    }

    #[test]
    fn test_u256_xor_self() {
        // x XOR x = 0
        let x = u64_to_u256(0x12345678);
        let result = u256_xor(&x, &x);
        assert!(u256_is_zero(&result));
    }

    #[test]
    fn test_u256_and_self() {
        // x AND x = x
        let x = u64_to_u256(0x12345678);
        let result = u256_and(&x, &x);
        assert_eq!(result, x);
    }

    #[test]
    fn test_u256_or_self() {
        // x OR x = x
        let x = u64_to_u256(0x12345678);
        let result = u256_or(&x, &x);
        assert_eq!(result, x);
    }

    #[test]
    fn test_u256_and_with_zero() {
        // x AND 0 = 0
        let x = u64_to_u256(0xFFFFFFFF);
        let result = u256_and(&x, &U256_ZERO);
        assert!(u256_is_zero(&result));
    }

    #[test]
    fn test_u256_or_with_zero() {
        // x OR 0 = x
        let x = u64_to_u256(0x12345678);
        let result = u256_or(&x, &U256_ZERO);
        assert_eq!(result, x);
    }

    #[test]
    fn test_u256_to_u64_overflow() {
        // Value with non-zero high bytes should return None
        let mut big = U256_ZERO;
        big[23] = 1; // Just outside the u64 range
        assert_eq!(u256_to_u64(&big), None);
    }

    #[test]
    fn test_u64_to_u256_roundtrip() {
        let values = [0u64, 1, 255, 256, 65535, u64::MAX];
        for val in values {
            let u256_val = u64_to_u256(val);
            assert_eq!(u256_to_u64(&u256_val), Some(val));
        }
    }
}
