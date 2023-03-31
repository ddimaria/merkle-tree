<!-- omit in toc -->
# Merkle Tree

This is the updated Merkle Tree that was created after the submission time was over.

<!-- omit in toc -->
## Contents

- [Running Tests](#running-tests)
- [Benchmarking](#benchmarking)
- [Documentation](#documentation)
  - [Create a new Merkle Tree](#create-a-new-merkle-tree)
  - [Retrieving the Root Hash](#retrieving-the-root-hash)
  - [Setting a Leaf Value](#setting-a-leaf-value)
  - [Generate a Proof](#generate-a-proof)
  - [Verify a Proof](#verify-a-proof)


## Running Tests

To run the unit tests:

```shell
git clone git@github.com:ddimaria/merkle-tree.git
cd merge-tree
cargo test
```

## Benchmarking

First install the dependencies:

```shell
brew install gnuplot
cargo install criterion
```

Now run the benchmarks:

```shell
cargo criterion
```

## Documentation

### Create a new Merkle Tree

> pub fn new(depth: usize, initial_leaf: Hash) -> MerkleTree

When creating a new Merkle Tree, an initial leaf node is chosen for all of the leaves.
The intermediate nodes and root node are calculated upon creation.
All nodes are hashed using `Sha3_256`.

```rust
use merkle_tree::MerkleTree, Proof;

let initial_leaf = MerkleTree::hash(&[0]);
let tree = MerkleTree::new(20, initial_leaf);
```

### Retrieving the Root Hash

> pub fn root(&self) -> Hash

After a Merkle Tree has been created, you can invoke the `root()` function to 
retrieve the root hash:

```rust
use merkle_tree::MerkleTree, Proof;

let initial_leaf = MerkleTree::hash(&[0]);
let tree = MerkleTree::new(20, initial_leaf);
let hash = tree::root();
```

### Setting a Leaf Value

> pub fn set(&mut self, offset: usize, value: Hash)

It's possible to set a leaf value after the tree has been created.  After 
setting the value, the affected hashes are recalculated.

```rust
use merkle_tree::MerkleTree, Proof;

let initial_leaf = MerkleTree::hash(&[0]);
let mut tree = MerkleTree::new(20, initial_leaf);

let new_leaf = MerkleTree::hash(&[1]);
tree.set(3, new_leaf).unwrap();
```

### Generate a Proof

> pub fn proof(&self, leaf: &Hash) -> Result<Proof>

A Merkle Proof contains the path from leaf to the root and all the sibling hash values along the way.

```rust
use merkle_tree::MerkleTree, Proof;
s
let initial_leaf = MerkleTree::hash(&[0]);
let tree = MerkleTree::new(2, initial_leaf);
let proof = tree.proof(&initial_leaf).unwrap();
```

### Verify a Proof

> pub fn verify(&self, proof: &Proof, data: &Hash) -> bool

The `verify()` function takes the proof and the leaf and verifies the proof against the tree's hash root.

```rust
use merkle_tree::MerkleTree, Proof;

let initial_leaf = MerkleTree::hash(&[0]);
let tree = MerkleTree::new(2, initial_leaf);
let proof = tree.proof(&initial_leaf).unwrap();
assert!(MerkleTree::verify(&proof, &initial_leaf));
```