# halo-kzg-cli

This is a [halo2-kzg](https://github.com/privacy-scaling-explorations/halo2) cli that can generate proofs targeting [acir](https://github.com/noir-lang/acvm/tree/master/acir).

Notice that at the moment you need to install [nargo](https://github.com/noir-lang/noir) first and run `nargo compile` to generate acir circuit.

## Installation

cd into this crate and run

```text
cargo install --path .
```

run

```text
halow_kzg --help
```

to see full functionalities.
