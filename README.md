CGGTTS 
======

Rust package to parse and generate CGGTTS data.

[![crates.io](https://img.shields.io/crates/v/cggtts.svg)](https://crates.io/crates/cggtts)
[![Rust](https://github.com/rtk-rs/cggtts/actions/workflows/rust.yml/badge.svg)](https://github.com/rtk-rs/cggtts/actions/workflows/rust.yml)
[![crates.io](https://docs.rs/cggtts/badge.svg)](https://docs.rs/cggtts/)
[![crates.io](https://img.shields.io/crates/d/cggtts.svg)](https://crates.io/crates/cggtts)    

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

## Set of tools

- `cggtts` is the main library. Compile it with the _scheduler_ option to unlock
full support of CGGTTS data production
- `cggtts-cli` is an application to analyze one or compare two CGGTTS files.  
Download its latest release from the [github portal](https://github.com/rtk-rs/cggtts/releases).

## Ecosystem

The CGGTTS solutions solver that is integrated to [the RINEX toolbox](https://github.com/georust/rinex)
is the _goto_ application to generate CGGTTS files from all this framework.  

The [RINEX Wiki pages](https://github.com/georust/rinex/wiki/CGGTTS) explain how you can resolve CGGTTS solutions
using this toolbox.

This crate heavily relies on `Hifitime` for accurate _Epoch_ representation
and _Timescales_ knowledge. Check out Christopher's amazing libraries [right here](https://github.com/nyx-space/hifitime).

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

## CGGTTS-CLI

[A command line application](gnss_cli/README.md) is developed to process one or two CGGTTS file for clock comparison.
