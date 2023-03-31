use criterion::{criterion_group, criterion_main, Criterion};

use merkle_tree::{MerkleTree, Proof};

fn bench_new() {
    let initial_leaf = MerkleTree::hash(&[0]);
    let _tree = MerkleTree::new(20, initial_leaf);
}

fn bench_set(tree: &mut MerkleTree, new_leaf: [u8; 32]) {
    tree.set(3, new_leaf);
}

fn bench_proof(tree: &mut MerkleTree, initial_leaf: &[u8; 32]) {
    let _proof = tree.proof(&initial_leaf).unwrap();
}

fn bench_verify(tree: &mut MerkleTree, initial_leaf: &[u8; 32], proof: &Proof) {
    assert!(tree.verify(&proof, &initial_leaf));
}

fn bench(c: &mut Criterion) {
    c.bench_function("bench_new", move |b| b.iter(|| bench_new()));

    let initial_leaf = MerkleTree::hash(&[0]);
    let tree = MerkleTree::new(20, initial_leaf);
    let mut tree_clone = tree.clone();

    c.bench_function("bench_set", move |b| {
        b.iter(|| bench_set(&mut tree_clone, initial_leaf))
    });

    let mut tree_clone = tree.clone();
    let new_leaf = MerkleTree::hash(&[1]);

    c.bench_function("bench_proof", move |b| {
        b.iter(|| bench_proof(&mut tree_clone, &new_leaf))
    });

    let mut tree_clone = tree.clone();
    let proof = tree.proof(&initial_leaf).unwrap();

    c.bench_function("bench_verify", move |b| {
        b.iter(|| bench_verify(&mut tree_clone, &new_leaf, &proof))
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
