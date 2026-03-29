//! BitReader for parsing variable-width bit fields from Victron data
//!
//! Victron protocol uses LSB-first bit ordering with variable-width fields
//! that are not byte-aligned. This module provides utilities to read these fields.

use crate::{Error, Result};

/// A reader for extracting variable-width bit fields from a byte slice
///
/// Bits are read LSB-first from each byte, matching the Victron protocol specification.
#[derive(Debug)]
pub struct BitReader<'a> {
    data: &'a [u8],
    byte_pos: usize,
    bit_pos: u8, // 0-7, position within current byte
}

impl<'a> BitReader<'a> {
    /// Create a new BitReader from a byte slice
    ///
    /// The reader starts at bit 0 of byte 0.
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            byte_pos: 0,
            bit_pos: 0,
        }
    }

    /// Read a single bit
    ///
    /// Returns true if the bit is 1, false if 0.
    pub fn read_bit(&mut self) -> Result<bool> {
        if self.byte_pos >= self.data.len() {
            return Err(Error::ParseError);
        }

        let byte = self.data[self.byte_pos];
        let bit = (byte >> self.bit_pos) & 1;

        self.advance(1)?;

        Ok(bit == 1)
    }

    /// Read an unsigned integer of specified bit width
    ///
    /// # Arguments
    /// * `bits` - Number of bits to read (1-64)
    ///
    /// # Returns
    /// The unsigned integer value
    pub fn read_unsigned_int(&mut self, bits: u8) -> Result<u64> {
        if bits == 0 || bits > 64 {
            return Err(Error::ParseError);
        }

        let mut value: u64 = 0;

        for i in 0..bits {
            if self.byte_pos >= self.data.len() {
                return Err(Error::ParseError);
            }

            let byte = self.data[self.byte_pos];
            let bit = (byte >> self.bit_pos) & 1;

            // LSB-first: first bit read goes to LSB of result
            value |= (bit as u64) << i;

            self.advance(1)?;
        }

        Ok(value)
    }

    /// Read a signed integer of specified bit width
    ///
    /// # Arguments
    /// * `bits` - Number of bits to read (1-64)
    ///
    /// # Returns
    /// The signed integer value (sign-extended)
    pub fn read_signed_int(&mut self, bits: u8) -> Result<i64> {
        if bits == 0 || bits > 64 {
            return Err(Error::ParseError);
        }

        let unsigned = self.read_unsigned_int(bits)?;

        // Sign extend
        let sign_bit = 1u64 << (bits - 1);
        if unsigned & sign_bit != 0 {
            // Negative number - extend sign bits
            let mask = !0u64 << bits;
            Ok((unsigned | mask) as i64)
        } else {
            Ok(unsigned as i64)
        }
    }

    /// Read an unsigned 8-bit integer
    pub fn read_u8(&mut self, bits: u8) -> Result<u8> {
        Ok(self.read_unsigned_int(bits)? as u8)
    }

    /// Read an unsigned 16-bit integer
    pub fn read_u16(&mut self, bits: u8) -> Result<u16> {
        Ok(self.read_unsigned_int(bits)? as u16)
    }

    /// Read an unsigned 32-bit integer
    pub fn read_u32(&mut self, bits: u8) -> Result<u32> {
        Ok(self.read_unsigned_int(bits)? as u32)
    }

    /// Read a signed 8-bit integer
    pub fn read_i8(&mut self, bits: u8) -> Result<i8> {
        Ok(self.read_signed_int(bits)? as i8)
    }

    /// Read a signed 16-bit integer
    pub fn read_i16(&mut self, bits: u8) -> Result<i16> {
        Ok(self.read_signed_int(bits)? as i16)
    }

    /// Read a signed 32-bit integer
    pub fn read_i32(&mut self, bits: u8) -> Result<i32> {
        Ok(self.read_signed_int(bits)? as i32)
    }

    /// Get the current byte position
    pub fn byte_position(&self) -> usize {
        self.byte_pos
    }

    /// Get the current bit position within the current byte
    pub fn bit_position(&self) -> u8 {
        self.bit_pos
    }

    /// Check if there are more bits to read
    pub fn has_bits(&self) -> bool {
        self.byte_pos < self.data.len()
    }

    /// Advance the read position by specified number of bits
    fn advance(&mut self, bits: u8) -> Result<()> {
        self.bit_pos += bits;

        while self.bit_pos >= 8 {
            self.bit_pos -= 8;
            self.byte_pos += 1;
        }

        Ok(())
    }
}
