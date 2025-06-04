# stegano_rs

**stegano_rs** is a Rust library for steganography. It provides various techniques for hiding and extracting data using low-level methods, such as bitplane manipulation and pixel value differencing (PVD).

Most methods and structures in this library operate on u8 array or byte slices, giving users the freedom to apply them not only to images, but also to any type of binary data such as audio, video, or custom formats.

## 📦 Features

This library offers multiple steganographic algorithms and flexible embedding locator strategies to control how and where data is hidden.

### Supported Algorithms

#### 🔹 Bitplane Methods
- `bitplane_embed`: general embedding using bitplanes.
- `bitplane_extract`: general extraction from bitplanes.
- `embed_lsb`: embed using the least significant bit.
- `extract_lsb`: extract from the least significant bit.
- `embed_msb`: embed using the most significant bit.
- `extract_msb`: extract from the most significant bit.

Configuration:
- [`BitplaneOptions`](src/bitplane.rs): options to customize bitplane-based embedding.

#### 🔹 Pixel Value Differencing (PVD)
- `pvd_embed`: embed data using pixel value differences.
- `pvd_extract`: extract data using pixel value differences.

Configuration:
- [`PvdOptions`](src/pvd.rs): options for PVD configuration.

## 🚀 Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
stegano-rs  = "0.1.0"
```

## 📘 Example
- Bitplane (LSB, MSB)

```rust
use stegano_rs::bitplane::{bitplane_embed, bitplane_extract, BitplaneOptions, embed_lsb, extract_lsb};
use stegano_rs::embedding_locator::{EmbeddingLocator, LinearTraversal};

fn main() {
    // Short message to hide
    let message = b"Hi"; // ASCII: [72, 105] = [0b01001000, 0b01101001]

    // 2 bytes * 8 bits = 16 bits to hide
    // We embed 2 bits per host byte -> need 8 host bytes
    let mut host = vec![0b1111_1111; 8]; // 8 bytes, all set to 0xFF

    // Bitplane configuration: use 2 bits, with LSB strategy
    let options = BitplaneOptions {
        bits_to_operate: 2,
        // Can be used default or custom embedding strategy
        embed_strategy: Some(embed_lsb), 
        extract_strategy: Some(extract_lsb),
        // ..BitplaneOptions::default() // Can be used to set other defaults
    };

    // Linear traversal over host : [0, ..., host.len() - 1]
    let locator = LinearTraversal;
    let indices: Vec<usize> = locator.iter_indices(host.len()).collect();

    // Embed the message into the host
    let embed_result = bitplane_embed(&mut host, message, &options, &indices);
    assert!(embed_result.is_ok(), "Embedding failed: {:?}", embed_result.err());

    // Extract the message back from the host
    let extracted = bitplane_extract(&host, &options, &indices);
    assert!(extracted.is_ok(), "Extraction failed: {:?}", extracted.err());

    let extracted = extracted.unwrap();

    // Check that the extracted message matches the original
    assert_eq!(extracted, message);
    println!("Message successfully embedded and extracted: {:?}", extracted); // [72, 105]
}
```
- PVD
```rust
use stegano_rs::pvd::{pvd_embed, pvd_extract, PvdOptions};
use stegano_rs::embedding_locator::{EmbeddingLocator, LinearTraversal};

fn main() {
    let secret_message = b"Hi"; // ASCII: [72, 105] = [0b01001000, 0b01101001]
    // Message to be hidden
    let mut host = vec![50, 80, 60, 100, 10, 50, 150, 210, 14, 58, 23, 47];

    let locator: LinearTraversal = LinearTraversal;
    let embedding_indices: Vec<usize> = locator.iter_indices(host.len()).collect();
    let options = PvdOptions::default();

    // Step 1: Embed the message
    let embed_result = pvd_embed(&mut host, secret_message, &options, &embedding_indices);
    assert!(embed_result.is_ok());
    assert_eq!(embed_result.unwrap(), secret_message.len() * 8);

    // Step 2: Extract the message
    let extract_result = pvd_extract(&host, &options, &embedding_indices);
    assert!(extract_result.is_ok());

    // The extracted message may contain extra data since no end marker is used.
    // We only verify that the extracted message starts with the original secret message.
    assert!(extract_result.clone().unwrap().starts_with(secret_message));
    println!(
        "Message successfully embedded and extracted: {:?}",
        extract_result.unwrap()
    ); // [72, 105, 12, 128]
}
```
For more examples, please read the documentation or the tests
## 🔍 Embedding Locators

Locators define how the data is traversed during the embedding process.

Available implementations of the [`EmbeddingLocator`](src/embedding_locator.rs) trait:
- `LinearTraversal`: simple sequential traversal.
- `PositionListTraversal`: custom list-based traversal.
- `HeatmapTraversal`: heatmap-based traversal for prioritized embedding regions.

## 🧱 Project Structure

Modules:
- `bitplane`: contains bitplane embedding/extraction functions and configuration.
- `pvd`: contains pixel value differencing methods.
- `embedding_locator`: defines traversal logic and strategies.

## 🤝 Contributions

Contributions are welcome! If you wish to improve **stegano_rs**, feel free to open a pull request on GitHub.

## 📄 License

This project is licensed under the MIT License. See the [`LICENSE`](./LICENSE) file for more details.
