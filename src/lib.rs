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
    /// Create a new MerkleTree.  Seed with all of the leaves. If there are an
    /// odd number of leaves, duplicate the last leaf and append to the vec.
    ///
    /// ```rust
    /// use merkle_tree::MerkleTree;
    /// let leaves = vec![MerkleTree::hash(b"a"), MerkleTree::hash(b"b")];
    /// let tree = MerkleTree::new(&leaves);
    /// ```
    pub fn new(leaves: &[Hash]) -> MerkleTree {
        let mut nodes = leaves.to_owned();

        // There are an odd number of leaves.  Duplicate the last leaf and
        // append to the vec.
        if nodes.len() % 2 == 1 {
            let last_leaf = nodes[nodes.len() - 1];
            nodes.extend_from_slice(&[last_leaf]);
        }

        let depth = Self::num_levels_from_leaves(&nodes);
        let mut last_index = nodes.len();

        // O(log n)
        for _i in 0..depth {
            let combined = nodes[0..last_index]
                .chunks_exact(2)
                .map(|chunk| Self::concat(&chunk[0], &chunk[1]))
                .collect::<Vec<Hash>>();

            last_index = combined.len();
            nodes = [combined, nodes].concat();
        }

        MerkleTree(nodes)
    }

    /// Update the value of an existing leaf and recalculate the root hash
    /// with only touching the affected nodes. O(log n) complexity in the loop.
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

    /// Return the hash root of the tree.
    pub fn root(&self) -> Hash {
        self.0[0]
    }

    /// Using the full size of the array, calculate the number of levels.
    pub fn num_levels(&self) -> usize {
        (self.0.len() as f32).log2().floor() as usize
    }

    /// Using the number of leaves, calculate the number of levels.
    /// This will always be an even number.
    /// This is zero-based, so a single level tree will have zero levels.
    pub fn num_levels_from_leaves(leaves: &Vec<Hash>) -> usize {
        (leaves.len() as f32).log2().floor() as usize
    }

    /// Using the position of a leaf, calcualte the array index.
    pub fn get_index_from_offset(&self, offset: usize) -> usize {
        let num_leaves = 2_usize.pow(self.num_levels() as u32);
        self.0.len() - num_leaves + offset
    }

    /// Get the array index of the parent node.
    pub fn get_parent_index(index: usize) -> usize {
        if index == 0 {
            0
        } else {
            index / 2 - ((index % 2) ^ 1)
        }
    }

    pub fn proof(&self, leaf: &Hash) -> Result<Proof> {
        let mut proof = Proof::new();

        // O(n)
        let mut position = self
            .0
            .iter()
            .position(|current_leaf| *current_leaf == *leaf)
            .ok_or_else(|| anyhow!("cannot find leaf {:?}", hex::encode(leaf)))?;

        // O(log n)
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
    fn get_the_parent_index() {
        assert_eq!(MerkleTree::get_parent_index(0), 0);
        assert_eq!(MerkleTree::get_parent_index(1), 0);
        assert_eq!(MerkleTree::get_parent_index(2), 0);
        assert_eq!(MerkleTree::get_parent_index(3), 1);
        assert_eq!(MerkleTree::get_parent_index(4), 1);
        assert_eq!(MerkleTree::get_parent_index(5), 2);
        assert_eq!(MerkleTree::get_parent_index(6), 2);
        assert_eq!(MerkleTree::get_parent_index(7), 3);
        assert_eq!(MerkleTree::get_parent_index(8), 3);
        assert_eq!(MerkleTree::get_parent_index(9), 4);
        assert_eq!(MerkleTree::get_parent_index(10), 4);
        assert_eq!(MerkleTree::get_parent_index(11), 5);
        assert_eq!(MerkleTree::get_parent_index(12), 5);
        assert_eq!(MerkleTree::get_parent_index(13), 6);
        assert_eq!(MerkleTree::get_parent_index(14), 6);
    }

    #[test]
    fn get_the_number_of_levels_from_leaves() {
        let leaves = leaves();
        assert_eq!(
            MerkleTree::num_levels_from_leaves(&leaves[0..0].to_vec()),
            0
        );
        assert_eq!(
            MerkleTree::num_levels_from_leaves(&leaves[0..2].to_vec()),
            1
        );
        assert_eq!(
            MerkleTree::num_levels_from_leaves(&leaves[0..4].to_vec()),
            2
        );
        assert_eq!(
            MerkleTree::num_levels_from_leaves(&leaves[0..6].to_vec()),
            2
        );
        assert_eq!(
            MerkleTree::num_levels_from_leaves(&leaves[0..8].to_vec()),
            3
        );
        assert_eq!(
            MerkleTree::num_levels_from_leaves(&leaves[0..10].to_vec()),
            3
        );
        assert_eq!(
            MerkleTree::num_levels_from_leaves(&leaves[0..12].to_vec()),
            3
        );
        assert_eq!(
            MerkleTree::num_levels_from_leaves(&leaves[0..14].to_vec()),
            3
        );
        assert_eq!(
            MerkleTree::num_levels_from_leaves(&leaves[0..16].to_vec()),
            4
        );
    }

    #[test]
    fn get_the_number_of_levels() {
        let leaves = leaves();
        assert_eq!(
            MerkleTree::num_levels(&MerkleTree::new(&leaves[0..0].to_vec())),
            0
        );
        assert_eq!(
            MerkleTree::num_levels(&MerkleTree::new(&leaves[0..1].to_vec())),
            1
        );
        assert_eq!(
            MerkleTree::num_levels(&MerkleTree::new(&leaves[0..2].to_vec())),
            1
        );
        assert_eq!(
            MerkleTree::num_levels(&MerkleTree::new(&leaves[0..3].to_vec())),
            2
        );
        assert_eq!(
            MerkleTree::num_levels(&MerkleTree::new(&leaves[0..4].to_vec())),
            2
        );
        assert_eq!(
            MerkleTree::num_levels(&MerkleTree::new(&leaves[0..5].to_vec())),
            3
        );
        assert_eq!(
            MerkleTree::num_levels(&MerkleTree::new(&leaves[0..6].to_vec())),
            3
        );
        assert_eq!(
            MerkleTree::num_levels(&MerkleTree::new(&leaves[0..7].to_vec())),
            3
        );
        assert_eq!(
            MerkleTree::num_levels(&MerkleTree::new(&leaves[0..8].to_vec())),
            3
        );
        assert_eq!(
            MerkleTree::num_levels(&MerkleTree::new(&leaves[0..9].to_vec())),
            4
        );
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
        // assert!(tree.verify(&proof, &old_leaf));

        // let new_leaf = MerkleTree::hash(b"c");
        // tree.set(15, new_leaf);
        // let new_root = tree.root();

        // // confirm that the hash root changed
        // assert_ne!(old_root, new_root);

        // let proof = tree.proof(&new_leaf).unwrap();
        // assert!(tree.verify(&proof, &new_leaf));
    }
}
