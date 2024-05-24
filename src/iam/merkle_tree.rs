use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use sha3::{Digest, Keccak256};

#[wasm_bindgen]
pub struct CompressedMerkleProof {
    path: Vec<bool>,
    hashes: Vec<Uint8Array>,
    leaf_index: u64,
}

#[wasm_bindgen]
impl CompressedMerkleProof {
    #[wasm_bindgen(constructor)]
    pub fn new(path: Vec<bool>, hashes: Vec<Uint8Array>, leaf_index: u64) -> CompressedMerkleProof {
        CompressedMerkleProof {
            path,
            hashes,
            leaf_index,
        }
    }

    pub fn verify(&self, root_hash: &Uint8Array, leaf_hash: &Uint8Array) -> bool {
        if self.path.len() != self.hashes.len() {
            return false;
        }

        let mut hash = leaf_hash.to_vec();
        for (i, bit) in self.path.iter().enumerate() {
            let sibling_hash = &self.hashes[i];
            hash = if *bit {
                MerkleTree::hash_combine(&hash, &sibling_hash.to_vec())
            } else {
                MerkleTree::hash_combine(&sibling_hash.to_vec(), &hash)
            };
        }

        hash == root_hash.to_vec()
    }
}

#[wasm_bindgen]
pub struct MerkleTree {
    levels: usize,
    next_index: u64,
    root: Uint8Array,
    filled_subtrees: Vec<Vec<Uint8Array>>,
    zero_values: Vec<Vec<Uint8Array>>,
    nodes: HashMap<String, Uint8Array>,
}

#[wasm_bindgen]
impl MerkleTree {
    #[wasm_bindgen(constructor)]
    pub fn new() -> MerkleTree {
        MerkleTree {
            levels: 0,
            next_index: 0,
            root: Uint8Array::new_with_length(32),
            filled_subtrees: Vec::new(),
            zero_values: Vec::new(),
            nodes: HashMap::new(),
        }
    }

    pub fn update(&mut self, leaf: &Uint8Array) -> u64 {
        let leaf_index = self.next_index;
        let mut path = Vec::new();
        let mut hash = leaf.to_vec();
        let mut current_node = hash.clone();

        while !current_node.is_empty() {
            let bit = current_node[current_node.len() - 1] == 1;
            path.push(bit);
            self.nodes.insert(path.iter().map(|&b| if b { '1' } else { '0' }).collect(), Uint8Array::from(&hash[..]));
            let sibling_path = MerkleTree::sibling_path(&path);
            let sibling_hash = self.nodes.get(&sibling_path.iter().map(|&b| if b { '1' } else { '0' }).collect()).cloned().unwrap_or_else(|| Uint8Array::new_with_length(32));
            hash = MerkleTree::hash_combine(&hash, &sibling_hash.to_vec());
            current_node = hash.clone();
        }

        self.next_index += 1;
        leaf_index
    }

    pub fn get_proof(&self, leaf_index: u64) -> CompressedMerkleProof {
        if leaf_index >= self.next_index {
            panic!("Leaf index out of bounds");
        }

        let mut path = Vec::new();
        let mut current_index = leaf_index;

        while current_index > 0 {
            let bit = current_index % 2 == 1;
            path.push(bit);
            current_index /= 2;
        }

        path.reverse();

        let mut hashes = Vec::new();
        for bit in &path {
            let sibling_path = MerkleTree::sibling_path(&path[..path.len() - 1]);
            let sibling_hash = self.nodes.get(&sibling_path.iter().map(|&b| if b { '1' } else { '0' }).collect()).cloned();
            if let Some(hash) = sibling_hash {
                hashes.push(hash);
            }
            path.pop();
        }

        hashes.reverse();

        CompressedMerkleProof::new(path, hashes, leaf_index)
    }

    fn sibling_path(path: &[bool]) -> Vec<bool> {
        let mut sibling_path = path.to_vec();
        let last_bit = sibling_path.pop().unwrap();
        sibling_path.push(!last_bit);
        sibling_path
    }

    fn hash_combine(left: &[u8], right: &[u8]) -> Vec<u8> {
        let mut hasher = Keccak256::new();
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().to_vec()
    }
}
