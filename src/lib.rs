pub mod error;

use error::{MerkleTreeError, Result};
use sha3::{Digest, Sha3_256};

#[derive(Debug)]
pub struct MerkleTree(Vec<Hash>);
pub type Hash = [u8; 32];
pub type Proof<'a> = Vec<(Direction, &'a Hash)>;

#[derive(Debug, PartialEq)]
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
    ///
    /// let leaves = [MerkleTree::hash(b"a"), MerkleTree::hash(b"b")];
    /// let tree = MerkleTree::new(&leaves).unwrap();
    /// ```
    pub fn new(leaves: &[Hash]) -> Result<MerkleTree> {
        if leaves.is_empty() {
            return Err(MerkleTreeError::Empty);
        }

        // take ownership of leaves
        let mut nodes = leaves.to_owned();

        // There are an odd number of leaves.  Duplicate the last leaf and
        // append to the vec.
        if nodes.len() % 2 == 1 {
            let last_leaf = nodes[nodes.len() - 1];
            nodes.extend_from_slice(&[last_leaf]);
        }

        let depth = Self::num_levels_from_leaves(&nodes);
        let mut last_index = nodes.len();

        // Add the concatenated branches to the end of the vec.  We can avoid
        // recursion since we can derive the number of levels in the tree.
        //
        // O(log n)
        for _i in 0..depth {
            let combined = nodes[0..last_index]
                .chunks_exact(2)
                .map(|chunk| Self::concat(&chunk[0], &chunk[1]))
                .collect::<Vec<Hash>>();

            last_index = combined.len();
            nodes = [combined, nodes].concat();
        }

        Ok(MerkleTree(nodes))
    }

    /// Update the value of an existing leaf and recalculate the root hash
    /// with only touching the affected nodes. O(log n) complexity in the loop.
    ///
    /// ```rust
    /// use merkle_tree::MerkleTree;
    ///
    /// let leaves = [MerkleTree::hash(b"a"), MerkleTree::hash(b"b")];
    /// let mut tree = MerkleTree::new(&leaves).unwrap();
    /// let old_leaf = leaves[1];
    /// let old_root = tree.root();
    ///
    /// let proof = tree.proof(&old_leaf).unwrap();
    /// assert!(tree.verify(&proof, &old_leaf));
    ///
    /// let new_leaf = MerkleTree::hash(b"c");
    /// tree.update(1, new_leaf).unwrap();
    /// let new_root = tree.root();
    ///
    /// // confirm that the hash root changed
    /// assert_ne!(old_root, new_root);
    ///
    /// let proof = tree.proof(&new_leaf).unwrap();
    /// assert!(tree.verify(&proof, &new_leaf));
    /// ```
    pub fn update(&mut self, offset: usize, value: Hash) -> Result<()> {
        if offset > self.num_leaves() - 1 {
            return Err(MerkleTreeError::OffsetOutOfBounds(
                offset,
                self.num_leaves(),
            ));
        }

        let mut position = self.get_index_from_offset(offset);
        let mut hash = value;

        // update the leaf's value
        self.0[position] = hash;

        // recalculate the hashes of the leaf's branch
        while position > 0 {
            hash = if position % 2 == 0 {
                Self::concat(&self.0[position - 1], &hash)
            } else {
                Self::concat(&hash, &self.0[position])
            };

            position = Self::get_parent_index(position);
            self.0[position] = hash;
        }

        Ok(())
    }

    /// Return the hash root of the tree.
    ///
    /// ```rust
    /// use merkle_tree::MerkleTree;
    ///
    /// let leaves = [MerkleTree::hash(b"a"), MerkleTree::hash(b"b")];
    /// let tree = MerkleTree::new(&leaves).unwrap();
    /// let expected = &MerkleTree::concat(&MerkleTree::hash(b"a"), &MerkleTree::hash(b"b"));
    /// assert_eq!(&tree.root(), expected);
    /// ```
    pub fn root(&self) -> Hash {
        self.0[0]
    }

    /// Using the full size of the array, calculate the number of levels.
    pub fn num_levels(&self) -> usize {
        Self::num_levels_from_leaves(&self.0)
    }

    /// Using the number of leaves, calculate the number of levels.
    /// This will always be an even number.
    /// This is zero-based, so a single level tree will have zero levels.
    pub fn num_levels_from_leaves(leaves: &[Hash]) -> usize {
        (leaves.len() as f32).log2().floor() as usize
    }

    /// Using the position of a leaf, calcualte the array index.
    pub fn get_index_from_offset(&self, offset: usize) -> usize {
        self.0.len() - self.num_leaves() + offset
    }

    /// Calculate the number of leaves in the tree from the number of levels.
    fn num_leaves(&self) -> usize {
        2_usize.pow(self.num_levels() as u32)
    }

    /// Get the array index of the parent node.
    pub fn get_parent_index(index: usize) -> usize {
        if index == 0 {
            0
        } else {
            index / 2 - ((index % 2) ^ 1)
        }
    }

    /// Generate a Merkle Proof for a given leaf.
    ///
    /// ```rust
    /// use merkle_tree::{MerkleTree, Direction};
    ///
    /// let leaves = [MerkleTree::hash(b"a"), MerkleTree::hash(b"b")];
    /// let leaf = leaves[1];
    /// let tree = MerkleTree::new(&leaves).unwrap();
    /// let proof = tree.proof(&leaf).unwrap();
    /// assert_eq!(proof, [(Direction::Left, &MerkleTree::hash(b"a"))]);
    /// ```
    pub fn proof(&self, leaf: &Hash) -> Result<Proof> {
        let mut proof = Proof::new();

        // O(n)
        // I tried out Rayon (par_iter().position_any()), but it was +69490%
        // slower than this approach.
        let mut position = self
            .0
            .iter()
            .position(|current_leaf| *current_leaf == *leaf)
            .ok_or_else(|| MerkleTreeError::CannotFindLeaf(hex::encode(leaf)))?;

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

    /// Verify a Merkle Proof for a given leaf.
    ///
    /// ```rust
    /// use merkle_tree::MerkleTree;
    ///
    /// let leaves = [MerkleTree::hash(b"a"), MerkleTree::hash(b"b")];
    /// let leaf = leaves[1];
    /// let tree = MerkleTree::new(&leaves).unwrap();
    /// let proof = tree.proof(&leaf).unwrap();
    /// assert!(tree.verify(&proof, &leaf));
    /// ```
    pub fn verify(&self, proof: &Proof, leaf: &Hash) -> bool {
        let root_hash = self.root();
        let mut current_hash = *leaf;

        for (hash_direction, hash) in proof.iter() {
            current_hash = match hash_direction {
                Direction::Left => Self::concat(hash, &current_hash),
                Direction::Right => Self::concat(&current_hash, hash),
            };
        }

        current_hash == root_hash
    }

    /// Hash a byte array.
    /// TODO(ddimaria): Allow the user to select/inject the hash function
    ///
    /// ```rust
    /// use merkle_tree::MerkleTree;
    ///
    /// let hash = MerkleTree::hash(b"a");
    /// assert_eq!(hash, [128, 8, 75, 242, 251, 160, 36, 117, 114, 111, 235, 44, 171, 45, 130, 21, 234, 177, 75, 198, 189, 216, 191, 178, 200, 21, 18, 87, 3, 46, 205, 139]);
    /// ```
    pub fn hash(data: &[u8]) -> Hash {
        Sha3_256::digest(data).into()
    }

    /// Concatenate
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
    fn leaves() -> [Hash; 16] {
        [
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
    fn root_hash(leaves: &[Hash]) -> Hash {
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
    fn it_returns_an_error_with_zero_leaves() {
        let tree = MerkleTree::new(&[]);
        assert!(tree.is_err());
    }

    #[test]
    fn gets_the_root_hash_of_even_leaves() {
        let leaves = leaves();
        let tree = MerkleTree::new(&leaves).unwrap();
        assert_eq!(tree.root(), root_hash(&leaves));
    }

    #[test]
    fn gets_the_root_hash_of_odd_leaves() {
        let leaves: &[Hash] = &leaves()[0..15];
        let tree = MerkleTree::new(&leaves).unwrap();

        // now that the tree is created, make the leaves even by coping the last leaf and compare
        let even_leaves = [leaves, &[leaves[14]]].concat();
        assert_eq!(tree.root(), root_hash(&even_leaves));
    }

    #[test]
    fn gets_the_parent_index() {
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
    fn gets_the_number_of_levels_from_leaves() {
        let leaves = leaves();
        assert_eq!(MerkleTree::num_levels_from_leaves(&[]), 0);
        assert_eq!(MerkleTree::num_levels_from_leaves(&leaves[0..0]), 0);
        assert_eq!(MerkleTree::num_levels_from_leaves(&leaves[0..2]), 1);
        assert_eq!(MerkleTree::num_levels_from_leaves(&leaves[0..4]), 2);
        assert_eq!(MerkleTree::num_levels_from_leaves(&leaves[0..6]), 2);
        assert_eq!(MerkleTree::num_levels_from_leaves(&leaves[0..8]), 3);
        assert_eq!(MerkleTree::num_levels_from_leaves(&leaves[0..10]), 3);
        assert_eq!(MerkleTree::num_levels_from_leaves(&leaves[0..12]), 3);
        assert_eq!(MerkleTree::num_levels_from_leaves(&leaves[0..14]), 3);
        assert_eq!(MerkleTree::num_levels_from_leaves(&leaves[0..16]), 4);
    }

    #[test]
    fn gets_the_number_of_levels() {
        let leaves = leaves();
        assert_eq!(
            MerkleTree::num_levels(&MerkleTree::new(&leaves[0..1]).unwrap()),
            1
        );
        assert_eq!(
            MerkleTree::num_levels(&MerkleTree::new(&leaves[0..2]).unwrap()),
            1
        );
        assert_eq!(
            MerkleTree::num_levels(&MerkleTree::new(&leaves[0..3]).unwrap()),
            2
        );
        assert_eq!(
            MerkleTree::num_levels(&MerkleTree::new(&leaves[0..4]).unwrap()),
            2
        );
        assert_eq!(
            MerkleTree::num_levels(&MerkleTree::new(&leaves[0..5]).unwrap()),
            3
        );
        assert_eq!(
            MerkleTree::num_levels(&MerkleTree::new(&leaves[0..6]).unwrap()),
            3
        );
        assert_eq!(
            MerkleTree::num_levels(&MerkleTree::new(&leaves[0..7]).unwrap()),
            3
        );
        assert_eq!(
            MerkleTree::num_levels(&MerkleTree::new(&leaves[0..8]).unwrap()),
            3
        );
        assert_eq!(
            MerkleTree::num_levels(&MerkleTree::new(&leaves[0..9]).unwrap()),
            4
        );
    }

    #[test]
    fn sets_a_leaf_value() {
        let leaves = leaves();
        let mut tree = MerkleTree::new(&leaves).unwrap();
        let old_leaf = leaves[15];
        let old_root = tree.root();

        let proof = tree.proof(&old_leaf).unwrap();
        assert!(tree.verify(&proof, &old_leaf));

        let new_leaf = MerkleTree::hash(b"c");
        tree.update(15, new_leaf).unwrap();
        let new_root = tree.root();

        // confirm that the hash root changed
        assert_ne!(old_root, new_root);

        let proof = tree.proof(&new_leaf).unwrap();
        assert!(tree.verify(&proof, &new_leaf));
    }

    #[test]
    fn errors_when_setting_a_non_existent_leaf() {
        let leaves = leaves();
        let mut tree = MerkleTree::new(&leaves).unwrap();
        let old_leaf = MerkleTree::hash(b"c");

        let proof = tree.proof(&old_leaf).unwrap();
        assert!(tree.verify(&proof, &old_leaf));

        let new_leaf = MerkleTree::hash(b"c");
        let result = tree.update(16, new_leaf);
        assert!(result.is_err());
    }

    #[test]
    fn gets_a_proof_and_verifies_for_all_leaves() {
        let leaves = leaves();
        let tree = MerkleTree::new(&leaves).unwrap();

        for i in 0..leaves.len() {
            let proof = tree.proof(&leaves[i]).unwrap();
            assert!(tree.verify(&proof, &leaves[i]));
        }
    }

    #[test]
    fn errors_when_getting_a_proof_for_a_non_existent_leaf() {
        let leaves = leaves();
        let tree = MerkleTree::new(&leaves).unwrap();
        let proof = tree.proof(&MerkleTree::hash(b"z"));

        assert!(proof.is_err());
    }

    #[test]
    fn does_not_verify_a_proof_for_a_non_existent_leaf() {
        let leaves = leaves();
        let tree = MerkleTree::new(&leaves).unwrap();
        let proof = tree.proof(&leaves[3]).unwrap();

        assert!(!tree.verify(&proof, &MerkleTree::hash(b"z")));
    }
}
