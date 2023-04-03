use anyhow::{anyhow, Result};
use sha3::{Digest, Sha3_256};

#[derive(Clone, Debug)]
pub struct MerkleTree(Vec<Hash>);
pub type Hash = [u8; 32];
pub type Proof<'a> = Vec<(Direction, &'a Hash)>;

#[derive(Debug)]
pub enum Direction {
    Left,
    Right,
}

impl MerkleTree {
    pub fn new(leaves: &[Hash]) -> MerkleTree {
        let mut nodes = leaves.to_owned();

        if nodes.len() % 2 == 1 {
            let last_leaf = nodes[nodes.len() - 1];
            nodes.extend_from_slice(&[last_leaf]);
        }

        let depth = Self::num_levels_from_leaves(&nodes);
        let mut children = nodes.clone();

        for _ in 0..depth {
            let combined = children
                .chunks_exact(2)
                .map(|chunk| Self::concat(&chunk[0], &chunk[1]))
                .collect::<Vec<Hash>>();

            children = combined.clone();
            nodes = [combined, nodes].concat();
        }

        MerkleTree(nodes)
    }

    pub fn set(&mut self, offset: usize, value: Hash) {
        let mut position = self.get_index_from_offset(offset);
        let mut hash = value;

        self.0[position] = hash;

        while position > 0 {
            hash = if position % 2 == 0 {
                Self::concat(&self.0[position - 1], &hash)
            } else {
                Self::concat(&hash, &self.0[position])
            };

            position = Self::get_parent_index(position);
            self.0[position] = hash;
        }
    }

    pub fn root(&self) -> Hash {
        self.0[0]
    }

    pub fn num_levels(&self) -> usize {
        (self.0.len() as f32).log2().floor() as usize
    }

    pub fn num_levels_from_leaves(leaves: &Vec<Hash>) -> usize {
        (leaves.len() as f32).log2().floor() as usize
    }

    pub fn get_index_from_offset(&self, offset: usize) -> usize {
        let num_leaves = 2_usize.pow(self.num_levels() as u32);
        self.0.len() - num_leaves + offset
    }

    pub fn get_parent_index(index: usize) -> usize {
        if index == 0 {
            0
        } else if index % 2 == 0 {
            index / 2 - 1
        } else {
            index / 2
        }
    }

    pub fn proof(&self, leaf: &Hash) -> Result<Proof> {
        let mut proof = Proof::new();
        let mut position = self
            .0
            .iter()
            .position(|current_leaf| *current_leaf == *leaf)
            .ok_or_else(|| anyhow!("cannot find leaf {:?}", hex::encode(leaf)))?;

        for _ in 0..self.num_levels() {
            let corresponding_hash = if position % 2 == 0 {
                (Direction::Left, &self.0[position - 1])
            } else {
                (Direction::Right, &self.0[position + 1])
            };

            proof.push(corresponding_hash);
            position = Self::get_parent_index(position);
        }

        Ok(proof)
    }

    pub fn verify(&self, proof: &Proof, data: &Hash) -> bool {
        let root_hash = self.root();
        let mut current_hash = *data;

        for (hash_direction, hash) in proof.iter() {
            current_hash = match hash_direction {
                Direction::Left => Self::concat(hash, &current_hash),
                Direction::Right => Self::concat(&current_hash, hash),
            };
        }

        current_hash == root_hash
    }

    pub fn hash(data: &[u8]) -> Hash {
        Sha3_256::digest(data).into()
    }

    pub fn concat(hash1: &Hash, hash2: &Hash) -> Hash {
        let mut combined = [0; 64];
        combined[..32].copy_from_slice(hash1);
        combined[32..].copy_from_slice(hash2);

        Self::hash(&combined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 16 leaves to use for tests
    fn leaves() -> Vec<Hash> {
        vec![
            MerkleTree::hash(b"a"),
            MerkleTree::hash(b"b"),
            MerkleTree::hash(b"c"),
            MerkleTree::hash(b"d"),
            MerkleTree::hash(b"e"),
            MerkleTree::hash(b"f"),
            MerkleTree::hash(b"g"),
            MerkleTree::hash(b"h"),
            MerkleTree::hash(b"i"),
            MerkleTree::hash(b"j"),
            MerkleTree::hash(b"k"),
            MerkleTree::hash(b"l"),
            MerkleTree::hash(b"m"),
            MerkleTree::hash(b"n"),
            MerkleTree::hash(b"o"),
            MerkleTree::hash(b"p"),
        ]
    }

    // 16 leaves to use for tests
    fn root_hash(leaves: &Vec<Hash>) -> Hash {
        MerkleTree::concat(
            &MerkleTree::concat(
                &MerkleTree::concat(
                    &MerkleTree::concat(&leaves[0], &leaves[1]),
                    &MerkleTree::concat(&leaves[2], &leaves[3]),
                ),
                &MerkleTree::concat(
                    &MerkleTree::concat(&leaves[4], &leaves[5]),
                    &MerkleTree::concat(&leaves[6], &leaves[7]),
                ),
            ),
            &MerkleTree::concat(
                &MerkleTree::concat(
                    &MerkleTree::concat(&leaves[8], &leaves[9]),
                    &MerkleTree::concat(&leaves[10], &leaves[11]),
                ),
                &MerkleTree::concat(
                    &MerkleTree::concat(&leaves[12], &leaves[13]),
                    &MerkleTree::concat(&leaves[14], &leaves[15]),
                ),
            ),
        )
    }

    #[test]
    fn gets_the_root_hash_of_even_leaves() {
        let leaves = leaves();
        let tree = MerkleTree::new(&leaves);
        assert_eq!(tree.root(), root_hash(&leaves));
    }

    #[test]
    fn gets_the_root_hash_of_odd_leaves() {
        let mut leaves = leaves()[0..15].to_vec();
        let tree = MerkleTree::new(&leaves);

        // now that the tree is created, make the leaves even by coping the last leaf and compare
        leaves.extend_from_slice(&[leaves[14]]);
        assert_eq!(tree.root(), root_hash(&leaves));
    }

    #[test]
    fn gets_a_proof_and_verifies_for_all_leaves() {
        let leaves = leaves();
        let tree = MerkleTree::new(&leaves);

        for i in 0..leaves.len() {
            let proof = tree.proof(&leaves[i]).unwrap();
            assert!(tree.verify(&proof, &leaves[i]));
        }
    }

    #[test]
    fn sets_a_leaf_value() {
        let leaves = leaves();
        let mut tree = MerkleTree::new(&leaves);
        let old_leaf = leaves[3];
        let old_root = tree.root();

        let proof = tree.proof(&old_leaf).unwrap();
        assert!(tree.verify(&proof, &old_leaf));

        let new_leaf = MerkleTree::hash(b"c");
        tree.set(15, new_leaf);
        let new_root = tree.root();

        // confirm that the hash root changed
        assert_ne!(old_root, new_root);

        let proof = tree.proof(&new_leaf).unwrap();
        assert!(tree.verify(&proof, &new_leaf));
    }
}
