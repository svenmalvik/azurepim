---
name: rust
description: Use this skill proactively when reviewing Rust code, writing new Rust code, or when the user asks about Rust best practices. Covers code style, ownership, async patterns, error handling, and performance.
---

# Rust Best Practices

## Tooling and Style
- Run `cargo fmt` and `cargo clippy --all-targets --all-features` before committing
- Fix all compiler warnings before committing - warnings indicate potential bugs
- Prefer `#![deny(clippy::all)]` at crate root
- Keep modules small; favor `pub(crate)` over `pub`
- Keep files to ~300 lines; extract submodules when larger
- Document public items with `///` and doctest examples

## Ownership and Data Structures
- Borrow by default; avoid `clone()` unless needed
- Prefer `&[T]`/`&str` over `Vec<T>`/`String` in APIs
- Use `Cow` when callers may pass borrowed or owned data
- Use `Arc` for shared ownership across threads
- Use `Option`/`Result` instead of sentinel values

## API Design
- Minimize type and lifetime complexity in public APIs
- Prefer `From`/`TryFrom` over custom constructors
- Use newtypes to add meaning to primitives
- Use builder pattern for configuration
- Return iterators where possible to avoid allocations

## Error Handling

### Library Code
- Define error enums with `thiserror`
- Avoid panics in library code

### Application Code
- Use `anyhow::Result` for ergonomic propagation
- Add context with `.with_context(...)`

### Rules
- Avoid `unwrap`/`expect` outside tests
- Prefer `?` operator for propagation
- Log errors once at the boundary

```rust
use anyhow::{Context, Result};

fn process_audio(path: &Path) -> Result<AudioData> {
    let file = std::fs::read(path)
        .with_context(|| format!("Failed to read audio file: {}", path.display()))?;
    parse_audio(&file).context("Failed to parse audio data")
}
```

## Async and Concurrency

### Runtime
- Use `tokio` as the async runtime
- Never block inside async code
- Use `spawn_blocking` for CPU-bound work

### Structured Concurrency
- Prefer channels (`mpsc`, `oneshot`) over manual threads
- Use `join!`, `try_join!` patterns
- Wrap shared state in `Arc<Mutex/RwLock>` only when necessary

### Timeouts and Cancellation
- Use timeouts for all async operations
- Propagate cancellation via `select!` or `tokio::time::timeout`

```rust
use tokio::time::{timeout, Duration};

async fn transcribe_with_timeout(audio: &[u8]) -> Result<String> {
    timeout(Duration::from_secs(30), call_stt_api(audio))
        .await
        .context("STT request timed out")?
}
```

### Observability
- Use `tracing` for structured logs
- Add spans around async work with `#[instrument]`
- Avoid logging sensitive data

```rust
#[instrument(skip(audio_data))]
async fn process_audio(meeting_id: &str, audio_data: &[u8]) -> Result<Transcript> {
    info!("Processing audio for meeting");
    // ...
}
```

## Testing
- Co-locate unit tests in modules (`mod tests { ... }`)
- Keep integration tests in `tests/`
- Use table-driven tests for coverage
- Add property-based tests for invariants
- Benchmark with `cargo bench` when performance matters

## Performance
- Measure before optimizing; use `cargo flamegraph`
- Prefer iterators over intermediate `Vec`s
- Reuse buffers; prefer small-copy types when justified
- Keep `unsafe` code isolated and well-tested

## Packaging
- Use workspaces for multiple crates
- Keep shared config in workspace `Cargo.toml`
- Move logic into libraries for testability
