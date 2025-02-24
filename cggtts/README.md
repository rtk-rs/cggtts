CGGTTS 
======

Rust library to parse and generate CGGTTS data.

[![crates.io](https://img.shields.io/crates/v/cggtts.svg)](https://crates.io/crates/cggtts)
[![Rust](https://github.com/rtk-rs/cggtts/actions/workflows/rust.yml/badge.svg)](https://github.com/rtk-rs/cggtts/actions/workflows/rust.yml)
[![crates.io](https://docs.rs/cggtts/badge.svg)](https://docs.rs/cggtts/badge.svg)
[![crates.io](https://img.shields.io/crates/d/cggtts.svg)](https://crates.io/crates/cggtts)    

CGGTTS is a file format to describe a local clock behavior against a single or the combination of clocks embedded in Satellite Vehicles (SV).  
Exchanging CGGTTS files enables so called "Common View" Time Transfer.

CGGTTS is specified by the Bureau International des Poids & des Mesures (BIPM):
[CGGTTS 2E specifications](https://www.bipm.org/documents/20126/52718503/G1-2015.pdf/f49995a3-970b-a6a5-9124-cc0568f85450)

This library only supports revision **2E**, and will _reject_ other revisions.

## Getting started 

Add "cggtts" to your Cargo.toml

```toml
cggtts = "4"
```

Use CGGTTS to parse local files

```rust
use cggtts::prelude::CGGTTS;

let cggtts = CGGTTS::from_file("../data/dual/GZGTR560.258");
assert!(cggtts.is_ok());

let cggtts = cggtts.unwrap();
assert_eq!(cggtts.station, "LAB");
assert_eq!(cggtts.tracks.len(), 2097);
```
