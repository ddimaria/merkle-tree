<!-- omit in toc -->
# Merkle Tree


TODO:
- [ ] Better documentation
- [ ] Error handling
- [ ] no_std feature
- [ ] Compare benchmarks with well-used Merkle Trees

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
use merkle_tree::MerkleTree;

let leaves = vec![MerkleTree::hash(b"a"), MerkleTree::hash(b"b")];
let tree = MerkleTree::new(&leaves);
```

### Retrieving the Root Hash

> pub fn root(&self) -> Hash

After a Merkle Tree has been created, you can invoke the `root()` function to 
retrieve the root hash:

```rust
use merkle_tree::MerkleTree;

let leaves = vec![MerkleTree::hash(b"a"), MerkleTree::hash(b"b")];
let tree = MerkleTree::new(&leaves);
let expected = &MerkleTree::concat(&MerkleTree::hash(b"a"), &MerkleTree::hash(b"b"));

assert_eq!(&tree.root(), expected);
```

### Setting a Leaf Value

> pub fn set(&mut self, offset: usize, value: Hash)

It's possible to set a leaf value after the tree has been created.  After 
setting the value, the affected hashes and the root hash are recalculated.

```rust
use merkle_tree::MerkleTree;

let leaves = vec![MerkleTree::hash(b"a"), MerkleTree::hash(b"b")];
let mut tree = MerkleTree::new(&leaves);
let old_leaf = leaves[1];
let old_root = tree.root();

let proof = tree.proof(&old_leaf).unwrap();
assert!(tree.verify(&proof, &old_leaf));

let new_leaf = MerkleTree::hash(b"c");
tree.set(1, new_leaf);
let new_root = tree.root();

// confirm that the hash root changed
assert_ne!(old_root, new_root);

let proof = tree.proof(&new_leaf).unwrap();
assert!(tree.verify(&proof, &new_leaf));
```

### Generate a Proof
> pub enum Direction { Left, Right }
> 
> pub type Proof<'a> = Vec<(Direction, &'a Hash)>;
> 
> pub fn proof(&self, leaf: &Hash) -> Result<Proof>

A Merkle Proof contains the path from leaf to the root and all the sibling hash values along the way.

```rust
use merkle_tree::{MerkleTree, Direction};

let leaves = vec![MerkleTree::hash(b"a"), MerkleTree::hash(b"b")];
let leaf = leaves[1];
let tree = MerkleTree::new(&leaves);
let proof = tree.proof(&leaf).unwrap();

assert_eq!(proof, vec![(Direction::Left, &MerkleTree::hash(b"a"))]);
```

### Verify a Proof

> pub fn verify(&self, proof: &Proof, leaf: &Hash) -> bool

The `verify()` function takes the proof and the leaf and verifies the proof against the tree's hash root.

```rust
use merkle_tree::MerkleTree;

let leaves = vec![MerkleTree::hash(b"a"), MerkleTree::hash(b"b")];
let leaf = leaves[1];
let tree = MerkleTree::new(&leaves);
let proof = tree.proof(&leaf).unwrap();

assert!(MerkleTree::verify(&proof, &leaf));
```