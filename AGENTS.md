# Contributor instructions

- Rust first. Keep OS integration behind narrow safe interfaces.
- Never add panic shortcuts or unsafe code to mobile-core.
- Use FilmCraft integer Tick time; validate external input and check arithmetic.
- Do not claim Android/iOS support from host tests alone.
- Preserve pinned upstream revisions and license notices.
- Never commit signing keys, local media, private paths, or environment secrets.
- Do not copy upstream trademark artwork or third-party assets without rights.
- Before committing: cargo fmt --all --check; cargo clippy --workspace --all-targets --locked -- -D warnings; cargo test --workspace --locked.
