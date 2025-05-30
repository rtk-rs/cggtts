CGGTTS 
======

Rust package to parse and generate CGGTTS data.

[![Rust](https://github.com/rtk-rs/cggtts/actions/workflows/rust.yml/badge.svg)](https://github.com/rtk-rs/cggtts/actions/workflows/rust.yml)
[![Rust](https://github.com/rtk-rs/cggtts/actions/workflows/daily.yml/badge.svg)](https://github.com/rtk-rs/cggtts/actions/workflows/daily.yml)
[![crates.io](https://docs.rs/cggtts/badge.svg)](https://docs.rs/cggtts/)
[![crates.io](https://img.shields.io/crates/d/cggtts.svg)](https://crates.io/crates/cggtts)

[![MRSV](https://img.shields.io/badge/MSRV-1.82.0-orange?style=for-the-badge)](https://github.com/rust-lang/rust/releases/tag/1.82.0)
[![License](https://img.shields.io/badge/license-MPL_2.0-orange?style=for-the-badge&logo=mozilla)](https://github.com/rtk-rs/sp3/blob/main/LICENSE)

## License

This library is part of the [RTK-rs framework](https://github.com/rtk-rs) which
is delivered under the [Mozilla V2 Public](https://www.mozilla.org/en-US/MPL/2.0) license.

## CGGTTS

CGGTTS is a file format designed to describe the state of a local clock with respect to spacecraft that belong
to GNSS constellation, ie., a GNSS timescale.  
Exchanging CGGTTS files allows direct clock comparison between two remote sites, by comparing how the clock behaves
with respect to a specific spacecraft (ie., on board clock).  
This is called the _common view_ time transfer technique. Although it is more accurate to say CGGTTS is just the comparison method,
what you do from the final results is up to end application. Usually, the final end goal is to have the B site track the A site
and replicate the remote clock. It is for example, one option to generate a UTC replica.

CGGTTS is specified by the Bureau International des Poids & des Mesures (BIPM):
[CGGTTS 2E specifications](https://www.bipm.org/documents/20126/52718503/G1-2015.pdf/f49995a3-970b-a6a5-9124-cc0568f85450)

This library only supports revision **2E**, and will _reject_ other revisions.

## Features

- `serdes`
- `scheduler`: unlock CGGTS track scheduling

## CGGTTS track scheduling

If you compiled the crate with the _scheduler_ feature, you can access the
`Scheduler` structure that helps you generate synchronous CGGTTS tracks.

Synchronous CGGTTS is convenient because it allows direct exchange of CGGTTS files
and therefore, direct remote clocks comparison.

The `Scheduler` structure works according to the BIPM definitions but we allow for a different
tracking duration. The default being 980s, you can use shorter tracking duration and faster
CGGTTS generation. You can only modify the tracking duration if you can do so on both remote clocks,
so they share the same production parameters at all times.

## System Time delays

A built in API allows accurate system delay description as defined in CGGTTS.

## Getting started

This library only supports revision **2E**, and will _reject_ other revisions.

Add "cggtts" to your Cargo.toml

```toml
cggtts = "4"
```

Use CGGTTS to parse local files

```rust
use cggtts::prelude::CGGTTS;

let cggtts = CGGTTS::from_file("data/CGGTTS/GZGTR560.258");
assert!(cggtts.is_ok());

let cggtts = cggtts.unwrap();
assert_eq!(cggtts.header.station, "LAB");
assert_eq!(cggtts.tracks.len(), 2097);
```
