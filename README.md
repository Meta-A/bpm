[![Code Coverage](https://github.com/theshpio/bpm/actions/workflows/code-coverage.yml/badge.svg)](https://github.com/theshpio/bpm/actions/workflows/code-coverage.yml)$${{\space}}$$
[![Coverage Status](https://coveralls.io/repos/github/Meta-A/bbpm/badge.svg)](https://coveralls.io/github/Meta-A/bbpm)
# BPM - Blockchain Package Manager

BPM is a cross-platform software that allows for verifiable, proven and secure package compilation, building, fetching, verification, and deployment.

![image](https://github.com/user-attachments/assets/c86a2e79-8384-4eb9-8513-d1b2f16a43ef)

# Notice

This project is still WIP, we are currently at an early stage where we are figuring out how to architecture and build BPM. If you would like to join us do not mind reaching out to us :)

# Clone repository

```sh
git clone --recurse-submodules git@github.com:Meta-A/bpm.git
```

# Dependencies

To build our project, you need to have the following dependencies installed:
- `pkg-config`
- `clang`
- `protobuf`

On Arch Linux, you can install them by running the following command:
```bash
pacman -S pkg-config clang protobuf
```

# Current properties & features
* Rust language
* Hashing of source code & binaries
* Hedera blockchain support
* ArchLinux PKGBUILD integration

# Planned features
* Support for 10-20 most popular packaging formats
* Support for several blockchains to choose from, including “multichain”
* BBPM running in a resident/daemon/service mode, which would allow auto scheduling
* Connections to “data storage” options: IPFS, Filecoin, etc.
* API, and several modules/samples using it
