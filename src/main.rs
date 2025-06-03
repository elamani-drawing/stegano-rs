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