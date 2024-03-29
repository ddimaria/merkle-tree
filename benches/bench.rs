use criterion::{criterion_group, criterion_main, Criterion};
use merkle_tree::{Hash, MerkleTree, Proof};

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

fn bench_new() {
    let _tree = MerkleTree::new(&leaves());
}

fn bench_update(tree: &mut MerkleTree, new_leaf: [u8; 32]) {
    tree.update(3, new_leaf).unwrap();
}

fn bench_proof(tree: &MerkleTree, leaf: &[u8; 32]) {
    let _proof = tree.proof(&leaf).unwrap();
}

fn bench_verify(tree: &MerkleTree, leaf: &[u8; 32], proof: &Proof) {
    assert!(tree.verify(&proof, &leaf));
}

fn bench(c: &mut Criterion) {
    c.bench_function("bench_new", |b| b.iter(|| bench_new()));

    let leaves = leaves();
    let leaf = leaves[15];
    let mut tree = MerkleTree::new(&leaves).unwrap();
    let new_leaf = MerkleTree::hash(b"z");

    c.bench_function("bench_set", |b| {
        b.iter(|| bench_update(&mut tree, new_leaf))
    });

    let mut tree = MerkleTree::new(&leaves).unwrap();

    c.bench_function("bench_proof", |b| b.iter(|| bench_proof(&mut tree, &leaf)));

    let tree = MerkleTree::new(&leaves).unwrap();
    let proof = tree.proof(&leaf).unwrap();

    c.bench_function("bench_verify", |b| {
        b.iter(|| bench_verify(&tree, &leaf, &proof))
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
