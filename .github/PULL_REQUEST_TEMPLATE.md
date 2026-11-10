## Summary

<!-- One paragraph: what changed and why. -->

## Type of change

- [ ] Bug fix (non-breaking)
- [ ] New feature (non-breaking)
- [ ] Breaking change
- [ ] Docs / chore

## Checklist

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo build --locked && cargo test --locked` passes
- [ ] `docker compose config --quiet` passes (if infra touched)
- [ ] `scripts/e2e.ps1` green (if backend touched)
- [ ] `CHANGELOG.md` `[Unreleased]` updated (Added/Changed/Fixed/Removed)
- [ ] Docs updated in the same PR
- [ ] Co-author trailers added to every commit (see `AGENTS.md`)

## Related

Closes # 