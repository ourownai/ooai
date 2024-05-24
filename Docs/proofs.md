# Compressed Merkle Proof

This Rust script demonstrates the implementation of a compressed Merkle proof format and its usage with the Solana service.

## Structs

### CompressedMerkleProof

The `CompressedMerkleProof` struct represents a compressed Merkle proof. It consists of the following fields:
- `path`: A vector of boolean values indicating the path to the leaf node.
- `hashes`: A vector of sibling hashes along the path.

The `CompressedMerkleProof` struct provides the following methods:
- `new`: Creates a new `CompressedMerkleProof` instance.
- `verify`: Verifies the compressed Merkle proof against a given root hash and leaf hash.
- `hash_combine`: Combines two hashes using the Keccak hash function.

### MerkleTree

The `MerkleTree` struct represents a sparse Merkle tree. It consists of the following field:
- `nodes`: A hashmap storing the non-empty nodes of the tree, with the path as the key and the hash value as the value.

The `MerkleTree` struct provides the following methods:
- `new`: Creates a new `MerkleTree` instance.
- `update`: Updates the Merkle tree with a new leaf.
- `get_proof`: Generates a compressed Merkle proof for a given leaf.
- `sibling_path`: Calculates the sibling path for a given path.
- `hash_combine`: Combines two hashes using the Keccak hash function.

## Usage

1. Make sure you have the Solana CLI installed and configured.

2. Save the Rust script in a file, for example, `merkle_proof.rs`.

3. Compile the code using the Solana CLI:
solana program build merkle_proof.rs

4. Deploy the compiled program to the Solana cluster:
solana program deploy target/deploy/merkle_proof.so

1. Invoke the `main` function of the deployed program:
solana program invoke <program_id> --url <cluster_url>

Replace `<program_id>` with the program ID obtained from the deployment step, and `<cluster_url>` with the URL of the Solana cluster you are using (e.g., `https://api.devnet.solana.com`).

1. Observe the output, which should display the root hash and the verification results for each leaf.

Note: Make sure you have sufficient SOL in your account to cover the transaction fees for deploying and invoking the program on the Solana cluster.

## Example

The `main` function in the script demonstrates the usage of the compressed Merkle proof format. It creates a sample Merkle tree with four leaves, updates the tree with each leaf, and retrieves the root hash. Then, it generates compressed Merkle proofs for each leaf and verifies the proofs against the root hash. The verification results are printed for each leaf.