# Collatz Sequence Implementations

<div align="center">
  <p align="center">
    <a href="http://makeapullrequest.com">
      <img alt="pull requests welcome badge" src="https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat">
    </a>
    <a href="https://x.com/m2magician">
      <img alt="Twitter" src="https://img.shields.io/twitter/url/https/twitter.com/m2magician.svg?style=social&label=Follow%20%40m2magician">
    </a>
  </p>
</div>

This repository contains implementations of the Collatz sequence using different backends: Plonky3 and Winterfell.

## Overview

The Collatz sequence is a sequence of numbers defined by the following rules:

1. Start with any positive integer n.
2. If n is even, divide it by 2.
3. If n is odd, multiply it by 3 and add 1.
4. Repeat the process with the result.

The Collatz conjecture states that for any positive integer n, the sequence will eventually reach 1.

This repo shows how to implement a proof of knowledge of a valid Collatz sequence using different frameworks.

Specifically, we prove that we know a sequence of numbers that is a valid Collatz sequence, and we know how many steps it takes to reach 1.

Public inputs:
- Starting value
- Length of the sequence


### plonky3-collatz
Implemented using the [Plonky3](https://github.com/Plonky3/Plonky3) backend for defining the AIR constraints. We use `p3-uni-stark` as the proving system in the example.

Run the example with:
```bash
cargo run -p plonky3-collatz
```

### winterfell-collatz
Implemented using the [Winterfell](https://github.com/facebook/winterfell) backend for defining the AIR constraints.

Run the example with:
```bash
cargo run -p winterfell-collatz
```


## Acknowledgments

The AIR constraints for the Collatz conjecture were originally described by StarkWare in the [Arithmetization I](https://medium.com/starkware/arithmetization-i-15c046390862) article. Those constraints, however, only attested to the knowledge of correct series of values in the Collatz sequence. Here, we also add the number of steps in that sequence as public input, and convince the verifier that the provided number of steps for a given starting value is correct. This is arguably more useful, for the rare occasion you want to convince someone of the validity of your Collatz computation.

Repo structure inspired by the [Fibonacci example](https://github.com/BrianSeong99/Plonky3_Fibonacci/) by @BrianSeong99, combined with the examples in the [Winterfell repo](https://github.com/facebook/winterfell/tree/main/examples).

## Disclaimer

This code is provided for experimental and educational purposes only. It has not been audited and should not be used in production environments or for any security-critical applications. Use at your own risk.
