# Blockchain qikfox

This project is a blockchain implementation written in Rust. It includes a main chain and a side chain, with features such as block creation, transaction validation, and consensus mechanisms.

## Table of Contents
- [Installation](#installation)
- [Usage](#usage)
- [Running Tests](#running-tests)
- [Project Structure](#project-structure)

## Installation

1. **Clone the repository:**
```bash

git clone https://github.com/knowpranjal/blockchain_1.git

cd blockchain_1

```

2. **Ensure that you have rust installed**
```bash

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

```

3. **Build the source**
```bash

cargo build

```

## Usage

1. **Run the blockchain**
```bash

cargo run

```

## Runnning Tests

1. **Run the tests**
```bash

cargo test

```

## Project Structure

```bash

├── src
│   ├── main.rs           # Entry point of the application
│   ├── blockchain.rs     # Blockchain logic
│   ├── block.rs          # Block logic 
│   └── genesis.rs        # Genesis block config
├── Cargo.toml            # Rust package manager file
└── README.md             # Project documentation

```
