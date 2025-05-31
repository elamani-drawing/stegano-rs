/// Configuration options for the Pixel Value Differencing (PVD) embedding method.
///
/// The `bins` field is a vector of tuples representing ranges of difference values.
/// Each bin defines a range `[min, max]` where differences falling within
/// that range will be used to embed a specific number of bits.
///
/// # Example bins
/// ```text
/// (0, 1), (2, 3), (4, 7), (8, 15), (16, 31), (32, 63), (64, 127), (128, 255)
/// ```
///
/// These bins are typical example ranges that determine the embedding capacity
/// depending on the pixel difference magnitude.
///
/// Note: These bins can be customized depending on the application and image characteristics.
pub struct PvdOptions {
    pub bins: Vec<(i32, i32)>,
}

impl Default for PvdOptions {
    /// Returns a default set of bins covering a range of difference values commonly used in PVD.
    ///
    /// The bins increase exponentially in size, allowing embedding of more bits as the pixel difference grows.
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
                diff.abs(), idx1, p1, idx2, p2
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_pvd_embed_success() {
        let mut host = vec![100u8, 110, 120, 130, 140, 150, 160, 170];
        let secret = b"A"; // 8 bits (1 bytes)
        let options = PvdOptions::default();
        let embedding_indices = vec![0, 7, 2, 3, 4, 5, 6, 7]; // 4 pairs of indices

        // Attempt to embed secret in host using PVD
        let result = pvd_embed(&mut host, secret, &options, &embedding_indices);
        // Expect success
        assert!(result.is_ok());
        // Expect all bits (16) to be embedded
        assert_eq!(result.unwrap(), 8);
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
}
