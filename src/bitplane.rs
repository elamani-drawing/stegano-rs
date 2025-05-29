/// Options for configuring bitplane-based steganography operations.
///
/// The `BitplaneOptions` struct encapsulates parameters that control how secret data
/// is embedded into or extracted from a host byte stream using bit-level operations.
///
/// # Fields
///
/// - `bits_to_operate`:  
///   Specifies how many bits of the secret message should be embedded or extracted per host byte.  
///   This must be a value between `1` and `8` (inclusive). Higher values increase
///   embedding capacity but may result in more noticeable changes in the host data.
///
/// - `embed_strategy`:  
///   An optional function pointer that defines the embedding strategy.  
///   The function takes three `u8` arguments:
///   1. The host byte.
///   2. The secret byte (or portion thereof).
///   3. The number of bits to operate.
///   
///   It returns a new `u8` that represents the host byte after embedding the secret bits.
///   If `None`, the default strategy (e.g., LSB) should be used.
///
/// - `extract_strategy`:  
///   An optional function pointer that defines the extraction strategy.  
///   The function takes two `u8` arguments:
///   1. The host byte.
///   2. The number of bits to operate.
///   
///   It returns a `u8` representing the extracted bits from the host byte.
///   If `None`, the default strategy (e.g., LSB) should be used.
///
/// # Example
/// ```rust
/// fn embed_lsb(host: u8, secret: u8, bits: u8) -> u8 {
///     let mask = (1 << bits) - 1;
///     (host & !mask) | (secret & mask)
/// }
///
/// fn extract_lsb(host: u8, bits: u8) -> u8 {
///     host & ((1 << bits) - 1)
/// }
///
/// //If you just want to embed, you don't need to enter extract, and vice versa
///
/// pub use stegano_rs::bitplane::BitplaneOptions;
/// let options = BitplaneOptions {
///     bits_to_operate: 2,
///     embed_strategy: Some(embed_lsb),
///     extract_strategy: Some(extract_lsb),
/// };
/// ```
#[derive(Debug, Clone, Copy)]
pub struct BitplaneOptions {
    /// The number of bits to operate per host byte (must be between 1 and 8).
    pub bits_to_operate: u8,

    /// Optional embedding strategy function.
    pub embed_strategy: Option<fn(u8, u8, u8) -> u8>,

    /// Optional extraction strategy function.
    
    /// Optional extraction strategy function.
    /// 
    /// This function takes a host byte and the number of bits to extract,
    /// and must return a `u8` where the extracted `bits_to_operate`
    /// are aligned to the **least significant bits** (i.e., right-aligned).
    ///
    /// This alignment is required for compatibility with the default `bitplane_extract` function.
    /// 
    /// Example: if `bits_to_operate = 3` and the embedded bits are `101`,
    /// the function must return `0b00000101`.
    pub extract_strategy: Option<fn(u8, u8) -> u8>,
}


impl Default for BitplaneOptions {
    /// Returns a default `BitplaneOptions` with:
    /// - `bits_to_operate` = 1,
    /// - `embed_strategy` = `embed_lsb`,
    /// - `extract_strategy` = `extract_lsb`.
    fn default() -> Self {
        Self {
            bits_to_operate: 1,
            embed_strategy: Some(embed_lsb), 
            extract_strategy: Some(extract_lsb),
        }
    }
}

/// Embed secret bits in the least significant bits of the host byte (LSB).
///
/// Strategy for BitplaneOptions
///
/// # Arguments
/// - `host_byte`: The byte from the host data to modify.
/// - `secret_bits`: The bits from the secret to embed.
/// - `bits`: Number of bits to embed.
///
/// # Returns
/// A new byte with the secret bits embedded in the least significant bits.
pub fn embed_lsb(host_byte: u8, secret_bits: u8, bits: u8) -> u8 {
    let mask = if bits == 8 { 0xFF } else { (1 << bits) - 1 };
    (host_byte & !mask) | (secret_bits & mask)
}

/// Embed secret bits in the most significant bits of the host byte (MSB).
///
/// Strategy for BitplaneOptions
///
/// # Arguments
/// - `host_byte`: The byte from the host data to modify.
/// - `secret_bits`: The bits from the secret to embed.
/// - `bits`: Number of bits to embed.
///
/// # Returns
/// A new byte with the secret bits embedded in the most significant bits.
pub fn embed_msb(host_byte: u8, secret_bits: u8, bits: u8) -> u8 {
    let mask = if bits == 8 {
        0xFF
    } else {
        ((1 << bits) - 1) << (8 - bits)
    };
    (host_byte & !mask) | (secret_bits << (8 - bits))
}


/// Extract secret bits embedded in the least significant bits (LSB) of the host byte.
/// 
/// # Arguments
/// - `host_byte`: The byte containing the embedded secret.
/// - `bits`: Number of bits embedded in the least significant bits.
/// 
/// # Returns
/// The extracted secret bits, aligned to the least significant bits.
pub fn extract_lsb(host_byte: u8, bits: u8) -> u8 {
    let mask = if bits == 8 { 0xFF } else { (1 << bits) - 1 };
    host_byte & mask
}

/// Extract secret bits embedded in the most significant bits (MSB) of the host byte.
/// 
/// # Arguments
/// - `host_byte`: The byte containing the embedded secret.
/// - `bits`: Number of bits embedded in the most significant bits.
/// 
/// # Returns
/// The extracted secret bits, shifted down to the least significant bits.
pub fn extract_msb(host_byte: u8, bits: u8) -> u8 {
    if bits == 8 {
        host_byte
    } else {
        host_byte >> (8 - bits)
    }
}

/// Embeds a secret message into a host buffer by modifying specific bits of each host byte
/// according to the provided bitplane embedding options.
///
/// # Arguments
///
/// * `host` - A mutable slice of bytes representing the host data where the secret will be embedded.
/// * `secret` - A slice of bytes containing the secret message to embed.
/// * `options` - A reference to a `BitplaneOptions` struct that configures the embedding process,
///               including the number of bits to modify per host byte and the embedding strategy function.
///
/// # Returns
///
/// Returns `Ok(())` if the embedding was successful, or an `Err(String)` describing the error.
/// # Example usage
///
/// ```rust
/// use stegano_rs::bitplane::{embed_lsb, bitplane_embed, BitplaneOptions};
/// let mut host_data = vec![0u8; 30];
/// let secret_message = b"hidden";
/// let options = BitplaneOptions {
///     bits_to_operate: 2,
///     embed_strategy: Some(embed_lsb),
///     extract_strategy: None,
/// };
///
/// bitplane_embed(&mut host_data, secret_message, &options).expect("Embedding failed");
/// ```
///
/// # Errors
///
/// The function returns an error string in the following cases:
/// - If `bits_to_operate` is not between 1 and 8.
/// - If no embedding strategy function is provided.
/// - If the host buffer is too small to embed the entire secret message.
///
/// # See also
///
/// `BitplaneOptions` struct, embedding strategy functions.
pub fn bitplane_embed(
    host: &mut [u8],
    secret: &[u8],
    options: &BitplaneOptions,
) -> Result<(), String> {
    // Validate bits_to_operate
    if options.bits_to_operate == 0 || options.bits_to_operate > 8 {
        return Err("options.bits_to_operate must be between 1 and 8".into());
    }
    // Validate embed_strategy
    let embed_fn = match options.embed_strategy {
        Some(f) => f,
        None => return Err("options.embed_strategy function must be provided".into()),
    };

    let total_bits = secret.len() * 8;
    let capacity = host.len() * options.bits_to_operate as usize;

    // Ensure there is enough space in the host to hide the secret
    if capacity < total_bits {
        return Err(format!(
            "Not enough space in host to hide the secret message: capacity={} bits, message={} bits",
            capacity, total_bits
        ));
    }

    let mut bit_index = 0;

    // Iterate over each host byte and embed bits
    for host_byte in host.iter_mut() {
        if bit_index >= total_bits {
            break;
        }

        // Extract up to `bits_to_operate` bits from the secret
        let mut secret_bits: u8 = 0;
        for i in 0..options.bits_to_operate {
            let bit_pos = bit_index + i as usize;
            if bit_pos >= total_bits {
                break;
            }

            let byte = secret[bit_pos / 8];
            let bit = (byte >> (7 - (bit_pos % 8))) & 1;
            secret_bits |= bit << (options.bits_to_operate - 1 - i);
        }

        // Apply the selected embedding strategy
        *host_byte = (embed_fn)(*host_byte, secret_bits, options.bits_to_operate);

        bit_index += options.bits_to_operate as usize;
    }

    Ok(())
}

/// Extracts a secret message from the host buffer using a bitplane extraction strategy.
/// 
/// # Arguments
/// - `host`: The byte buffer containing the embedded secret.
/// - `options`: The options indicating how to extract the secret, including the number
///   of bits to operate on and the extraction strategy function.
/// 
/// # Returns
/// A vector of bytes representing the extracted secret.
/// 
/// # Requirements
/// - `options.bits_to_operate` must be between 1 and 8.
/// - `options.extract_strategy` must be provided and must return a `u8` where
///   the extracted bits are aligned to the least significant bits (right-aligned).
///   For example, if `bits_to_operate = 3` and the extracted bits are `101`,
///   the function should return `0b00000101`.
/// 
/// # Example
/// ```rust
/// use stegano_rs::bitplane::{extract_lsb, bitplane_extract, BitplaneOptions};
/// let mut host_data = vec![0u8; 10];
/// let secret = bitplane_extract(&host_data, &BitplaneOptions {
///     bits_to_operate: 3,
///     extract_strategy: Some(extract_lsb),
///     embed_strategy: None,
/// });
/// ```
pub fn bitplane_extract(
        host: &[u8],
        options: &BitplaneOptions,
    ) -> Result<Vec<u8>, String> {
        
    // Validate bits_to_operate
    if options.bits_to_operate == 0 || options.bits_to_operate > 8 {
        return Err("options.bits_to_operate must be between 1 and 8".into());
    }

    // Get the extraction function
    let extract_fn = match options.extract_strategy {
        Some(f) => f,
        None => return Err("No extract strategy provided".into()),
    };
    
    // Estimate the maximum number of bits we can extract from the host
    let total_bits = host.len() * options.bits_to_operate as usize;

    // Compute how many full bytes that corresponds to
    let total_bytes = (total_bits + 7) / 8;
    let mut secret = vec![0u8; total_bytes];
    
    let mut bit_index = 0;

    // Iterate over each byte in the host buffer
    for host_byte in host.iter() {
        // Extract only the bits_to_operate bits using the strategy
        let extracted_bits = (extract_fn)(*host_byte, options.bits_to_operate);

        // Go through each bit extracted from the current host byte
        for i in 0..options.bits_to_operate {
            // Extract the bit at position i
            let bit = (extracted_bits >> (options.bits_to_operate - 1 - i)) & 1;

            // Calculate the index in the secret buffer
            let byte_index = bit_index / 8;
            // Calculate the bit offset within that byte
            let bit_offset = 7 - (bit_index % 8);

            // Ensure we don't write out of bounds
            // We shift the bit to its correct position without erasing the others already written
            if byte_index < secret.len() {
                secret[byte_index] |= bit << bit_offset;
            }

            bit_index += 1;
        }
    }

    Ok(secret)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Bitplane embedding strategy tests
    #[test]
    fn test_embed_lsb() {
        // host_byte = 0b11110000, secret_bits = 0b00000011, bits = 2
        let result: u8 = embed_lsb(0b11110000, 0b00000011, 2);
        // Replace the 2 least significant bits with 11 -> expected result: 0b11110011 (243)
        assert_eq!(result, 0b11110011);

        // With bits = 4, secret_bits = 0b1010
        let result: u8 = embed_lsb(0b00001111, 0b00001010, 4);
        // Result: least significant 4 bits replaced by 1010 -> 0b00001010 (10)
        assert_eq!(result, 0b00001010);

        // bits = 8, replace all bits
        let result: u8 = embed_lsb(0b00000000, 0b10101010, 8);
        assert_eq!(result, 0b10101010);
    }

    #[test]
    fn test_embed_msb() {
        // host_byte = 0b00001111, secret_bits = 0b00000011, bits = 2
        let result = embed_msb(0b00001111, 0b00000011, 2);
        // Replace the 2 most significant bits with 11 -> expected result: 0b11001111 (207)
        assert_eq!(result, 0b11001111);

        // bits = 4, secret_bits = 0b1010
        let result = embed_msb(0b11110000, 0b00001010, 4);
        // Result: most significant 4 bits replaced by 1010 -> 0b10100000 (160)
        assert_eq!(result, 0b10100000);

        // bits = 8, replace all bits
        let result = embed_msb(0b11111111, 0b01010101, 8);
        assert_eq!(result, 0b01010101);
    }


    #[test]
    fn test_embed_success() {
        let mut host = vec![255, 255, 255, 255]; // 4 bytes host
        let secret = vec![0b1010_1100]; // 8 bits secret
        let options = BitplaneOptions {
            bits_to_operate: 2,
            embed_strategy: Some(embed_lsb),
            extract_strategy: None,
        };

        let res = bitplane_embed(&mut host, &secret, &options);
        assert!(res.is_ok());

        let expected = vec![254, 254, 255, 252];
        assert_eq!(host, expected);
    }

    #[test]
    fn test_bits_to_operate_zero_error() {
        let mut host = vec![0u8; 10];
        let secret = vec![0u8; 1];
        let options = BitplaneOptions {
            bits_to_operate: 0,
            embed_strategy: Some(embed_lsb),
            extract_strategy: None,
        };

        let res = bitplane_embed(&mut host, &secret, &options);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            "options.bits_to_operate must be between 1 and 8"
        );
    }

    #[test]
    fn test_bits_to_operate_greater_than_8_error() {
        let mut host = vec![0u8; 10];
        let secret = vec![0u8; 1];
        let options = BitplaneOptions {
            bits_to_operate: 9,
            embed_strategy: Some(embed_lsb),
            extract_strategy: None,
        };

        let res = bitplane_embed(&mut host, &secret, &options);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            "options.bits_to_operate must be between 1 and 8"
        );
    }

    #[test]
    fn test_no_embed_strategy_error() {
        let mut host = vec![0u8; 10];
        let secret = vec![0u8; 1];
        let options = BitplaneOptions {
            bits_to_operate: 2,
            embed_strategy: None,
            extract_strategy: None,
        };

        let res = bitplane_embed(&mut host, &secret, &options);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            "options.embed_strategy function must be provided"
        );
    }

    #[test]
    fn test_insufficient_capacity_error() {
        let mut host = vec![0u8; 1]; // 1 byte host
        let secret = vec![0u8; 2]; // 16 bits secret
        let options = BitplaneOptions {
            bits_to_operate: 2,
            embed_strategy: Some(embed_lsb),
            extract_strategy: None,
        };

        // Capacity = 1 * 2 = 2 bits < 16 bits of secret, should error
        let res = bitplane_embed(&mut host, &secret, &options);
        assert!(res.is_err());
        assert!(
            res.unwrap_err()
                .starts_with("Not enough space in host to hide the secret message")
        );
    }

    // Bitplane extraction strategy tests
    
    #[test]
    fn test_extract_lsb() {
        // Extract 1 bit from LSB (should be 1)
        assert_eq!(extract_lsb(0b0000_0001, 1), 0b1);

        // Extract 3 bits from LSB (should be 0b101)
        assert_eq!(extract_lsb(0b0000_0101, 3), 0b101);

        // Extract 5 bits from LSB (should be 0b10101)
        assert_eq!(extract_lsb(0b0001_0101, 5), 0b10101);

        // Extract 8 bits from LSB (whole byte)
        assert_eq!(extract_lsb(0b1010_1010, 8), 0b1010_1010);
    }

    #[test]
    fn test_extract_msb() {
        // Extract 1 bit from MSB (should be 1)
        assert_eq!(extract_msb(0b1000_0000, 1), 0b1);

        // Extract 3 bits from MSB (should be 0b101)
        assert_eq!(extract_msb(0b1010_0000, 3), 0b101);

        // Extract 5 bits from MSB (should be 0b10101)
        assert_eq!(extract_msb(0b1010_1000, 5), 0b10101);

        // Extract 8 bits from MSB (whole byte)
        assert_eq!(extract_msb(0b0101_0101, 8), 0b0101_0101);
    }

    // Bitplane extraction tests
    
    #[test]
    fn test_extract_lsb_simple() {
        // Host bytes with 3 secret bits embedded in LSB: 0b00000101 (bits=101) and 0b00000011 (bits=011)
        let host = [
            0b00000101,
            0b00000011,
        ];
        let options = BitplaneOptions {
            bits_to_operate: 3,
            extract_strategy: Some(extract_lsb),
            ..BitplaneOptions::default()
        };

        let secret = bitplane_extract(&host, &options).unwrap();

        // Expected extracted bits: 101 011 -> 0b10101100 (last 2 bits padded with 0)
        assert_eq!(secret.len(), 1);
        assert_eq!(secret[0], 0b10101100);
    }

    #[test]
    fn test_extract_msb_simple() {
        // Host bytes with 3 secret bits embedded in MSB: 0b10100000 (bits=101) and 0b01100000 (bits=011)
        let host = [
            0b10100000,
            0b01100000,
        ];
        let options = BitplaneOptions {
            bits_to_operate: 3,
            extract_strategy: Some(extract_msb),
            ..BitplaneOptions::default()
        };

        let secret = bitplane_extract(&host, &options).unwrap();

        // Expected extracted bits: 101 011 -> 0b10101100
        assert_eq!(secret.len(), 1);
        assert_eq!(secret[0], 0b10101100);
    }

    #[test]
    fn test_extract_full_byte() {
        // Host bytes with full bytes embedded in LSB (8 bits)
        let host = [
            0b10101010,
            0b11001100,
        ];
        let options = BitplaneOptions {
            bits_to_operate: 8,
            extract_strategy: Some(extract_lsb),
            ..BitplaneOptions::default()
        };

        let secret = bitplane_extract(&host, &options).unwrap();

        // Each host byte is fully extracted as is
        assert_eq!(secret.len(), 2);
        assert_eq!(secret[0], 0b10101010);
        assert_eq!(secret[1], 0b11001100);
    }
}
