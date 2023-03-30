use anyhow::{anyhow, Result};
use sha3::{Digest, Sha3_256};

pub struct MerkleTree(Vec<Vec<Hash>>);
pub type Hash = [u8; 32];
pub type Proof<'a> = Vec<(Direction, &'a Hash)>;

#[derive(Debug)]
pub enum Direction {
    Left,
    Right,
}

// TODO: add guards
pub fn get_index(depth: usize, offset: usize) -> usize {
    2_usize.pow(depth as u32) + offset - 1
}

// TODO: add guards
pub fn get_depth_offset(index: usize) -> (usize, usize) {
    let depth = ((index + 1) as f32).log2().floor() as usize;
    let offset = index - get_index(depth, 0);

    (depth, offset)
}

// TODO: add guards
pub fn get_parent_index(index: usize) -> usize {
    index / 2
}

// TODO: add guards
pub fn get_index_left_child(index: usize) -> usize {
    index * 2
}

impl MerkleTree {
    pub fn new(depth: usize, initial_leaf: Hash) -> MerkleTree {
        let bottom = vec![initial_leaf; 2 * depth];
        let mut nodes: Vec<Vec<Hash>> = vec![bottom];
        let mut current_level = 0;

        if nodes[0].len() % 2 == 1 {
            let last_leaf = nodes[0][nodes[0].len() - 1];
            nodes[0].append(&mut vec![last_leaf]);
        }

        while nodes[current_level].len() > 1 {
            let combined = nodes[current_level]
                .chunks_exact(2)
                .map(|chunk| Self::concat(&chunk[0], &chunk[1]))
                .collect::<Vec<Hash>>();

            nodes.append(&mut vec![combined]);
            current_level += 1;
        }

        MerkleTree(nodes)
    }

    pub fn root(&self) -> Hash {
        self.0[self.0.len() - 1][0]
    }

    pub fn num_levels(&self) -> usize {
        self.0.len()
    }

    fn leaves(&self) -> &Vec<Hash> {
        &self.0[0]
    }

    pub fn proof(&self, leaf: &Hash) -> Result<Proof> {
        let mut proof = Proof::new();
        let mut position = self
            .leaves()
            .iter()
            .position(|current_leaf| *current_leaf == *leaf)
            .ok_or_else(|| anyhow!("cannot find leaf {:?}", leaf))?;

        for level in 0..self.num_levels() - 1 {
            let corresponding_hash = if position % 2 == 0 {
                (Direction::Right, &self.0[level][position + 1])
            } else {
                (Direction::Left, &self.0[level][position - 1])
            };

            proof.push(corresponding_hash);
            position /= 2;
        }

        Ok(proof)
    }

    pub fn verify(proof: &Proof, data: &Hash, root_hash: &Hash) -> bool {
        let mut current_hash = *data;

        for (hash_direction, hash) in proof.iter() {
            current_hash = match hash_direction {
                Direction::Left => Self::concat(hash, &current_hash),
                Direction::Right => Self::concat(&current_hash, hash),
            };
        }

        current_hash == *root_hash
    }

    pub fn hash(data: &[u8]) -> Hash {
        Sha3_256::digest(data).into()
    }

    pub fn concat(hash1: &Hash, hash2: &Hash) -> Hash {
        // let combined = [hash1, hash2].concat();
        let mut combined = [0; 64];
        combined[..32].copy_from_slice(hash1);
        combined[32..].copy_from_slice(hash2);

        Self::hash(&combined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_gets_an_index() {
        // TODO: make a loop
        let result = get_index(0 as usize, 0 as usize);
        assert_eq!(result, 0);

        let result = get_index(1 as usize, 0 as usize);
        assert_eq!(result, 1);

        let result = get_index(1 as usize, 1 as usize);
        assert_eq!(result, 2);

        let result = get_index(2 as usize, 0 as usize);
        assert_eq!(result, 3);
    }

    #[test]
    fn it_gets_depth_offset() {
        // todo: make a loop
        let result = get_depth_offset(0);
        assert_eq!(result, (0, 0));

        let result = get_depth_offset(1);
        assert_eq!(result, (1, 0));

        let result = get_depth_offset(2);
        assert_eq!(result, (1, 1));

        let result = get_depth_offset(3);
        assert_eq!(result, (2, 0));
    }

    #[test]
    fn it_gets_the_parent_index() {
        // todo: make a loop
        let result = get_parent_index(0);
        assert_eq!(result, 0);

        let result = get_parent_index(1);
        assert_eq!(result, 0);

        let result = get_parent_index(2);
        assert_eq!(result, 1);

        let result = get_parent_index(3);
        assert_eq!(result, 1);
    }

    #[test]
    fn gets_the_index_left_child() {
        // todo: make a loop
        let result = get_index_left_child(0);
        assert_eq!(result, 0);

        let result = get_index_left_child(1);
        assert_eq!(result, 2);

        let result = get_index_left_child(2);
        assert_eq!(result, 4);

        let result = get_index_left_child(3);
        assert_eq!(result, 6);
    }

    #[test]
    fn gets_the_root_hash() {
        let initial_leaf = MerkleTree::hash(&[0]);
        let tree = MerkleTree::new(20, initial_leaf);
        assert_eq!(
            hex::encode(tree.root()),
            "4d7f3122e5024215635044db229fa7942b256b98838656c74f416cfdc309ee64"
        );
    }
    #[test]
    fn gets_a_proof_and_verifies() {
        let initial_leaf = MerkleTree::hash(&[0]);
        let tree = MerkleTree::new(2, initial_leaf);
        let proof = tree.proof(&initial_leaf).unwrap();
        assert!(MerkleTree::verify(&proof, &initial_leaf, &tree.root()));
    }
}
