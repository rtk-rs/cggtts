CGGTTS cli 
==========

Command line tool to analyze and parse CGGTTS data

[![crates.io](https://img.shields.io/crates/v/cggtts-cli.svg)](https://crates.io/crates/cggtts-cli)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/rtk-rs/cggtts/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/rtk-rs/cggtts/blob/main/LICENSE-MIT) 
[![crates.io](https://img.shields.io/crates/d/cggtts.svg)](https://crates.io/crates/cggtts)    
[![Rust](https://github.com/rtk-rs/cggtts/actions/workflows/rust.yml/badge.svg)](https://github.com/rtk-rs/cggtts/actions/workflows/rust.yml)
[![crates.io](https://docs.rs/cggtts/badge.svg)](https://docs.rs/cggtts/badge.svg)


## Command line interface

`--filepath` or `-f` is the only mandatory argument.
This argument accepts a list of local file:

```shell
cggtts-cli --filepath data/advanced/RZSY8257.000 --bipm-compliant --header
cggtts-cli -f data/advanced/RZSY8257.000,data/standard/GZSY8259.506 --bipm-compliant
```

All other arguments are optionnal.  
Refer to `help` menu for more information
