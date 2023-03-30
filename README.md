# Merkle Tree

_NOTE: I didn't quite finish this, but got some implemented, but not enough refactor time to get this into a single level vec and add the set function.  I'm going to finish in a branch to keep this branch fixed._



## Running Tests

```shell
git clone git@github.com:ddimaria/merkle-tree.git
cd merge-tree
cargo test
```

## Documentation

## Create a new Merkle Tree

```rust
let initial_leaf = MerkleTree::hash(&[0]);
let tree = MerkleTree::new(20, initial_leaf);
```

## Generate a Proof

```rust
let initial_leaf = MerkleTree::hash(&[0]);
let tree = MerkleTree::new(2, initial_leaf);
let proof = tree.proof(&initial_leaf).unwrap();
```

## Verify a Proof

```rust
let initial_leaf = MerkleTree::hash(&[0]);
let tree = MerkleTree::new(2, initial_leaf);
let proof = tree.proof(&initial_leaf).unwrap();
assert!(MerkleTree::verify(&proof, &initial_leaf, &tree.root()));
```