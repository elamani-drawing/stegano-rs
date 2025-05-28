/// Options for configuring bitplane-based steganography operations.
///
/// The `BitplaneOptions` struct encapsulates parameters that control how secret data
/// is embedded into a host byte stream using bit-level operations.
///
/// # Fields
///
/// - `bits_to_hide`:  
///   Specifies how many bits of the secret message should be hidden in each host byte.  
///   This must be a value between `1` and `8` (inclusive). Higher values increase
///   embedding capacity but may result in more noticeable changes in the host data.
///
/// - `strategy`:  
///   A function pointer defining the embedding strategy.  
///   The function takes three `u8` arguments:
///   1. The host byte.
///   2. The secret byte (or portion thereof).
///   3. The number of bits to hide.
///   
///   It returns a new `u8` that represents the host byte after embedding the secret bits.
///
/// # Example
/// ```rust
/// fn lsb_strategy(host: u8, secret: u8, bits: u8) -> u8 {
///     let mask = (1 << bits) - 1;
///     (host & !mask) | (secret & mask)
/// }
///
/// let options = BitplaneOptions {
///     bits_to_hide: 2,
///     strategy: lsb_strategy,
/// };
/// ```
#[derive(Debug, Clone, Copy)]
pub struct BitplaneOptions {
    /// The number of bits to hide per host byte (must be between 1 and 8).
    pub bits_to_hide: u8,

    /// The strategy function that defines how to embed secret bits into a host byte.
    pub strategy: fn(u8, u8, u8) -> u8,
}
