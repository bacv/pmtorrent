# Poor -Man's- Torrent

_Toy example of file chunking and sharing in tamperproof way using Merkle tree_

The essential part of this project is a MerkleTree trait that has a default implementation for the majority of it's functions. The tree it self can be build from anything that can be hashed with the provided hasher and represented as bytes. 

## Merkle tree

If a slice of leaves are provided as `[l1, l2, l3, l4]`, the returned tree will have such layout:
```
[
	hash_l1,
	hash_l2,
	hash_l3, 
	hash_l4, 
	hash_parent_l1_l2, 
	hash_parent_l3_l4, 
	hash_root
]
```
The storage of a tree is left for the implementor to figure out. As of this example, everything is stored in a memory by the `pmtorrent::File` struct.
The methods for calculating varios tree properties such as node count, sibling or parent indexes are basically the same as for Heap data structure, the only modifiaction is that all indexes had to be calculated in reverse, because in this design the root node of a tree is at the end of a vector.

## How to build

To build the project run:
```bash
cargo build
```

## How to run

The project has only one binary that can chunk the provided file and serve it over http. To run:
```bash
cargo run -- /path/to/the/file.to_chunk
```

