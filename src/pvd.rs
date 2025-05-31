/// Configuration options for the Pixel Value Differencing (PVD) embedding method.
///
/// The `bins` field is a vector of tuples representing ranges of pixel difference values.
/// Each bin defines a range `[min, max]` in which a pixel difference must fall to embed
/// a certain number of bits. The embedding capacity increases with the size of the bin,
/// enabling more bits to be embedded in image regions with higher variation.
///
/// # Default Bins and Embedding Capacity
///
/// ```text
/// (0, 1)     → 1 bit
/// (2, 3)     → 1 bit
/// (4, 7)     → 2 bits
/// (8, 15)    → 3 bits
/// (16, 31)   → 4 bits
/// (32, 63)   → 5 bits
/// (64, 127)  → 6 bits
/// (128, 255) → 7 bits
/// ```
///
/// These bins are chosen to balance **imperceptibility** and **capacity**:
/// - Small differences hide fewer bits to reduce visible artifacts.
/// - Larger differences hide more bits where variations are already visible.
///
/// # Customization
///
/// You may override these bins depending on the image type and desired robustness.
/// For smoother images (e.g., portraits), prefer smaller bins to preserve quality.
/// For complex images (e.g., landscapes), larger bins can be used for higher capacity.
///
/// # Example
/// ```rust
/// use stegano_rs::pvd::PvdOptions;
/// let options = PvdOptions::default(); // use default bin setup
/// for (min, max) in &options.bins {
///     println!("Bin {:?}-{:?} can hide {} bits",
///         min, max, ((max - min + 1) as f64).log2().floor() as usize);
/// }
/// ```
///
/// # Security Note
///
/// This method is not cryptographically secure on its own. For secure data hiding,
/// consider encrypting your data before embedding it with PVD.
///
/// # See also
///
/// - `pvd_embed()`
/// - `pvd_extract()`
pub struct PvdOptions {
    pub bins: Vec<(i32, i32)>,
}

impl Default for PvdOptions {
    /// Returns a default set of bins covering a range of difference values commonly used in PVD.
    ///
    /// The bins increase exponentially in size, allowing embedding of more bits
    /// as the pixel difference grows. This adaptive embedding is key to maintaining
    /// visual fidelity while maximizing capacity.
    fn default() -> Self {
        Self {
            bins: vec![
                (0, 1),
                (2, 3),
                (4, 7),
                (8, 15),
                (16, 31),
                (32, 63),
                (64, 127),
                (128, 255),
            ],
        }
    }
}


/// Embeds a secret message into a host buffer using the Pixel Value Differencing (PVD) technique.
///
/// This function ignores the last pixel if it does not form a complete pair with another pixel.
///
/// # Arguments
/// - `host`: Mutable slice of host data bytes where the secret will be embedded.
/// - `secret`: Slice of bytes representing the secret message to embed.
/// - `options`: Reference to `PvdOptions` that configures the embedding process, including bins.
/// - `embedding_indices`: Slice of indices indicating positions in the host where pairs of bytes are taken.
///
/// # Returns
/// Returns the number of embedded bits on success, or an error string otherwise.
///
/// # Example
/// ```rust
/// use stegano_rs::pvd::{pvd_embed, PvdOptions};
/// let mut host = vec![100u8, 110, 120, 130, 140, 150, 160, 170];
/// let secret = b"A";
/// let options = PvdOptions::default();
/// let embedding_indices = vec![0, 1, 2, 3, 4, 5, 6, 7];
/// let bits_embedded = pvd_embed(&mut host, secret, &options, &embedding_indices).unwrap();
/// ```
pub fn pvd_embed(
    host: &mut [u8],
    secret: &[u8],
    options: &PvdOptions,
    embedding_indices: &[usize],
) -> Result<usize, String> {
    // Check that the bin configuration is not empty
    if options.bins.is_empty() {
        return Err("options.bins cannot be empty".into());
    }

    let total_secret_bits = secret.len() * 8; // Total number of bits in the secret message
    let mut bit_index = 0; // Number of bits embedded so far

    // Iterate over the embedding indices two-by-two to form pixel pairs
    for pair in embedding_indices.chunks(2) {
        if pair.len() < 2 {
            break; // If we have an incomplete pair, stop
        }

        if bit_index >= total_secret_bits {
            break; // All bits have been embedded
        }

        let idx1 = pair[0];
        let idx2 = pair[1];

        // Ensure indices are within the bounds of the host buffer
        if idx1 >= host.len() || idx2 >= host.len() {
            continue; // Skip invalid index pairs
        }

        let p1 = host[idx1] as i32;
        let p2 = host[idx2] as i32;
        let diff = p1 - p2;

        // Find the appropriate bin range for the current difference
        let bin_option = options
            .bins
            .iter()
            .find(|&&(min_bin, max_bin)| diff.abs() >= min_bin && diff.abs() <= max_bin);

        if bin_option.is_none() {
            return Err(format!(
                "Difference {} at positions idx1={} (pixel: {}) and idx2={} (pixel: {}) does not fit any bin",
                diff.abs(),
                idx1,
                p1,
                idx2,
                p2
            ));
        }

        let (min_bin, max_bin) = *bin_option.unwrap();
        let range_size = (max_bin - min_bin + 1) as usize;

        // Number of bits we can hide in this bin
        let bits_to_embed = (range_size as f64).log2().floor() as usize;

        // Extract bits_to_embed bits from the secret starting at bit_index
        let mut secret_bits = 0u32;
        let mut actual_bits = 0;
        for i in 0..bits_to_embed {
            let global_bit_pos = bit_index + i;
            if global_bit_pos >= total_secret_bits {
                break;
            }
            let byte = secret[global_bit_pos / 8];
            let bit = (byte >> (7 - (global_bit_pos % 8))) & 1;
            secret_bits = (secret_bits << 1) | (bit as u32);
            actual_bits += 1;
        }

        if actual_bits == 0 {
            break; // Plus de bits à insérer
        }

        // Calculate the new difference value using the extracted bits
        let new_diff_sign = if diff >= 0 { 1 } else { -1 };
        let new_diff = min_bin + secret_bits as i32;

        // Recompute pixel values so their difference equals new_diff, preserving average
        let avg = (p1 + p2) / 2;
        let new_p1 = avg + (new_diff_sign * ((new_diff + 1) / 2));
        let new_p2 = avg - (new_diff_sign * (new_diff / 2));

        // Ensure new pixel values are valid (in range 0..=255)
        if new_p1 < 0 || new_p1 > 255 || new_p2 < 0 || new_p2 > 255 {
            continue; // Skip if pixel overflow would occur
        }

        // Update host pixels
        host[idx1] = new_p1 as u8;
        host[idx2] = new_p2 as u8;

        bit_index += actual_bits; // Increment only by the number of bits actually embedded
    }

    // Final check: if not all secret bits were embedded, return error
    if bit_index < total_secret_bits {
        return Err(format!(
            "Not enough capacity to embed the full secret: embedded {}/{} bits",
            bit_index, total_secret_bits
        ));
    }

    Ok(bit_index)
}

/// Extracts the hidden secret message from a host buffer using the Pixel Value Differencing (PVD) technique.
///
/// This function processes pairs of pixels indexed by `extraction_indices` and reconstructs the secret bits
/// embedded according to the bin configuration in `options`. The last pixel is ignored if it does not form a complete pair.
///
/// # Arguments
/// - `host`: Slice of host data bytes containing the embedded secret.
/// - `options`: Reference to `PvdOptions` that configures the extraction process, including bins.
/// - `extraction_indices`: Slice of indices indicating positions in the host where pairs of bytes are taken.
///
/// # Returns
/// Returns a `Vec<u8>` containing the extracted secret bytes on success, or an error string otherwise.
///
/// # Example
/// ```rust
/// use stegano_rs::pvd::{pvd_extract, PvdOptions};
/// let host = vec![100u8, 110, 120, 130, 140, 150, 160, 170];
/// let options = PvdOptions::default();
/// let extraction_indices = vec![0, 1, 2, 3, 4, 5];
/// let secret = pvd_extract(&host, &options, &extraction_indices).unwrap();
/// ```
pub fn pvd_extract(
    host: &[u8],
    options: &PvdOptions,
    extraction_indices: &[usize],
) -> Result<Vec<u8>, String> {
    // Check that the bin configuration is not empty
    if options.bins.is_empty() {
        return Err("options.bins cannot be empty".into());
    }

    let mut extracted_bytes = Vec::new();
    let mut current_byte = 0u8;
    let mut bits_in_current_byte = 0;

    // Iterate over the extraction indices two-by-two to form pixel pairs
    for pair in extraction_indices.chunks(2) {
        if pair.len() < 2 {
            break; // Ignore last pixel if it doesn't form a pair
        }

        let idx1 = pair[0];
        let idx2 = pair[1];

        // Skip invalid indices
        if idx1 >= host.len() || idx2 >= host.len() {
            continue;
        }

        let p1 = host[idx1] as i32;
        let p2 = host[idx2] as i32;

        let diff = p1 - p2;
        let diff_abs = diff.abs();

        // Find the bin corresponding to the absolute difference
        let bin_option = options
            .bins
            .iter()
            .find(|&&(min_bin, max_bin)| diff_abs >= min_bin && diff_abs <= max_bin);

        let (min_bin, max_bin) = match bin_option {
            Some(b) => *b,
            None => {
                return Err(format!(
                    "Difference {} at positions idx1={} (pixel: {}) and idx2={} (pixel: {}) does not fit any bin",
                    diff_abs, idx1, p1, idx2, p2
                ));
            }
        };

        let range_size = (max_bin - min_bin + 1) as usize;

        // Calculate the number of bits encoded in this bin
        let bits_to_extract = (range_size as f64).log2().floor() as usize;

        // Extract the hidden value from the difference
        let hidden_value = (diff_abs - min_bin) as u32;

        // Extract bits from the hidden value starting from the most significant bit
        for i in (0..bits_to_extract).rev() {
            let bit = ((hidden_value >> i) & 1) as u8;

            // Shift current_byte to the left and add the extracted bit
            current_byte = (current_byte << 1) | bit;
            bits_in_current_byte += 1;

            // Once we have 8 bits, push the byte to the output vector
            if bits_in_current_byte == 8 {
                extracted_bytes.push(current_byte);
                current_byte = 0;
                bits_in_current_byte = 0;
            }
        }
    }

    // If the last byte is not fully filled with bits, pad with zeros on the right
    if bits_in_current_byte > 0 {
        current_byte <<= 8 - bits_in_current_byte;
        extracted_bytes.push(current_byte);
    }

    Ok(extracted_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    // embed tests
    #[test]
    fn test_pvd_embed_success() {
        let mut host = vec![100u8, 110, 120, 130, 140, 150, 160, 170];
        let secret = b"A"; // 8 bits (1 bytes)
        let options = PvdOptions::default();
        let embedding_indices = vec![0, 1, 2, 3, 4, 5, 6, 7]; // 4 pairs of indices

        // Attempt to embed secret in host using PVD
        let result = pvd_embed(&mut host, secret, &options, &embedding_indices);
        // Expect success
        assert!(result.is_ok());
        // Expect all bits (16) to be embedded
        assert_eq!(result.unwrap(), 8);
    }

    #[test]
    fn test_pvd_embed_odd_number_of_indices() {
        let mut host = vec![100u8, 110, 120, 130, 140, 150, 160, 170, 140];
        let secret = b"A"; // 8 bits (1 bytes)
        let options = PvdOptions::default();
        let embedding_indices = vec![0, 1, 2, 3, 4, 5, 6, 7, 8]; 

        let result = pvd_embed(&mut host, secret, &options, &embedding_indices);
        assert!(result.is_ok());

        let bits_embedded = result.unwrap();

        // Check that the last pixel was ignored (idx 8)
        assert_eq!(host[8], 140u8);

        // bits_embedded should be at least 3 (for the first pair)
        assert!(bits_embedded >= 3);
    }

    #[test]
    fn test_pvd_embed_failure_not_enough_space() {
        let mut host = vec![100u8, 110]; // only one pair available
        let secret = b"AB"; // 2*8 bits (2 byte)
        let options = PvdOptions::default();
        let embedding_indices = vec![0, 1];

        // Attempt to embed secret in a too small host buffer
        let result = pvd_embed(&mut host, secret, &options, &embedding_indices);

        // Expect an error due to insufficient capacity
        assert!(result.is_err());
        println!("result: {:?}", result);
        assert!(result.err().unwrap().contains("Not enough capacity"));
    }

    #[test]
    fn test_pvd_embed_error_empty_bins() {
        let mut host = vec![100u8, 110];
        let secret = b"X";
        let options = PvdOptions { bins: vec![] };
        let indices = vec![0, 1];
        let result = pvd_embed(&mut host, secret, &options, &indices);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("bins cannot be empty"));
    }

    #[test]
    fn test_pvd_embed_error_diff_not_in_any_bin() {
        let mut host = vec![10u8, 250]; // diff = 240
        let secret = b"!";
        let options = PvdOptions {
            bins: vec![(0, 1), (2, 3)], // does not include diff = 240
        };
        let indices = vec![0, 1];

        let result = pvd_embed(&mut host, secret, &options, &indices);
        assert!(result.is_err());
    }

    // extract tests

    #[test]
    fn test_pvd_extract_empty_bins() {
        let host = vec![100, 110, 120, 130];
        let options = PvdOptions { bins: vec![] };
        let indices = vec![0, 1, 2, 3];

        let result = pvd_extract(&host, &options, &indices);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "options.bins cannot be empty");
    }

    #[test]
    fn test_pvd_extract_difference_not_in_bin() {
        // Bins that do not cover difference of 20
        let options = PvdOptions {
            bins: vec![(0, 5), (6, 10)],
        };
        let host = vec![50, 30]; // difference = 20
        let indices = vec![0, 1];

        let result = pvd_extract(&host, &options, &indices);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not fit any bin"));
    }

    #[test]
    fn test_pvd_extract_success() {
        // Setup bins so difference = 4 fits (0..7)
        let options = PvdOptions { bins: vec![(0, 7)] };
        // host pixels chosen so diff = p1 - p2 = 4
        // diff_abs = 4 fits bin 0..7, bits_to_extract = floor(log2(8))=3 bits
        let host = vec![120, 116];
        let indices = vec![0, 1];

        let secret = pvd_extract(&host, &options, &indices).unwrap();

        // We expect 3 bits extracted from hidden value = diff_abs - min_bin = 4
        // Binary 4 = 100 (3 bits)
        // So extracted bytes will be these bits padded on the right to 8 bits: 10000000 = 0x80
        assert_eq!(secret, vec![0x80]);
    }

    #[test]
    fn test_pvd_extract_multiple_pairs() {
        // Bins 0..7 with 3 bits per pair
        let options = PvdOptions { bins: vec![(0, 7)] };
        // Two pairs:
        // pair 1: diff = 5 => bits = 101
        // pair 2: diff = 3 => bits = 011
        let host = vec![130, 125, 140, 137];
        let indices = vec![0, 1, 2, 3];

        let secret = pvd_extract(&host, &options, &indices).unwrap();

        // bits concatenated: 101 011 => 6 bits
        // padded to 8 bits: 10101100 = 0xAC
        assert_eq!(secret, vec![0xAC]);
    }

    #[test]
    fn test_pvd_extract_odd_number_of_indices() {
        let options = PvdOptions { bins: vec![(0, 7)] };
        // Host with 3 pixels (odd number)
        let host = vec![130, 125, 140];
        // indices with odd length
        let indices = vec![0, 1, 2]; // last index '2' should be ignored

        let result = pvd_extract(&host, &options, &indices);
        assert!(result.is_ok());

        let extracted = result.unwrap();
        // Only first two pixels used: diff = 130 - 125 = 5 (fits bin 0..7)
        // bits extracted = 3 bits, binary 5 = 101, padded to byte = 10100000 = 0xA0
        assert_eq!(extracted, vec![0xA0]);
    }
}
