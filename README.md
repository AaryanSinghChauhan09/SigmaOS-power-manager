# SigmaOS Power Manager

This crate contains power-profile and thermal-state models. For example,
`PowerManager::set_profile()` changes in-memory frequency fields, and thermal
updates can change those model values. The crate does not read battery or
thermal sensors or program CPU/GPU hardware from this model API.

Treat these values as simulated policy state until a platform backend and
hardware evidence are documented. The repository depends on the sibling
`SigmaOS-kernel` checkout through a local path.

## License

GPL-3.0-or-later. See [`LICENSE`](LICENSE) and [`Cargo.toml`](Cargo.toml).
