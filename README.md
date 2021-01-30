# serde-binprot
This crates provides [binprot](https://github.com/janestreet/bin_prot)
serialization via [serde](https://github.com/serde-rs/serde). It tries
to provide the same serialization as the OCaml version for similar types.

This is an early prototype, and would need a lot more testing.
Known limitations include:

- Only enum with less than 256 variants are supported. This is because serde does not provide a way to know the total number of variants for an enum, see [serde#663](https://github.com/serde-rs/serde/issues/663). 
