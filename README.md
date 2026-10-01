<p align="center">
  <img src="docs/media/logo-128.png" alt="TidyID" height="64">
</p>

<p align="center">
  <a href="https://crates.io/crates/tidyid"><img src="https://img.shields.io/crates/v/tidyid.svg" alt="crates.io version"></a>
  <a href="https://docs.rs/tidyid"><img src="https://img.shields.io/docsrs/tidyid" alt="docs.rs"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.85%2B-93450a.svg?logo=rust" alt="Rust 1.85+"></a>
  <a href="https://github.com/sheldonix/tidyid-rust/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

A secure, high-performance, and human-friendly ID generator for Rust.

- **Secure** Uses the platform CSPRNG with unbiased sampling and no weak fallback. Generate independently across threads, processes, and cluster nodes; enforce absolute uniqueness with a database constraint.
- **High-performance** Optimized for fast, low-overhead ID generation and efficient scaling across concurrent workloads.
- **Human-friendly** Creates letter-first, lowercase-alphanumeric IDs by default with a fixed `LLD` rhythm—no accidental long words, punctuation, or ambiguous characters. Easy to read, type, and transcribe; ready for URLs, filenames, database/cache/object-storage keys, DOM/CSS IDs, command lines, logs, and more.

```rust
use tidyid::tidyid;

let id1 = tidyid(32, false)?; // jh4yp2fm6rq6hj4fe6xh9aq8ar5ax3ng (length = 32)
let id2 = tidyid(16, false)?; // ce6qy8gc9nd7uu8r                 (length = 16)
let id3 = tidyid(10, false)?; // yj7tf9xw2r                       (length = 10)
let id4 = tidyid(10, true)?;  // bQ3pB7gG5C                       (length = 10, allow_uppercase = true)
```

## Install

```sh
cargo add tidyid
```

**or** add it to `Cargo.toml`:

```toml
[dependencies]
tidyid = "2.1.1"
```

Enable the `metrics` feature to use the capacity and entropy helpers:

```toml
[dependencies]
tidyid = { version = "2.1.1", features = ["metrics"] }
```

## CLI

Install the command-line binary:

```sh
cargo install tidyid
```

Generate IDs:

```sh
tidyid
# dz6ut2ff4fq3vx2br2kw9qd3tg4xb9hn (length = 32)

tidyid -s 16
# gj3gp4kp7uv4tb2k (length = 16)

tidyid -s 10 -u
# Wx7pQ9jA2F (length = 10, allow_uppercase = true)
```

Use `--size` or `-s` to set the length. Use `--allow-uppercase` or `-u`
to allow uppercase letters.

## Format

By default, IDs repeat two lowercase letters followed by one digit (`LLD`):

```text
xr3 fc9 xy2
```

| Characters | Positions | Alphabet |
| --- | --- | --- |
| Letters | First two of each group | `abcdefghjkmnpqrtuvwxyz` |
| Digits | Every third character | `23456789` |

- Every ID starts with a letter.
- `i`, `l`, `o`, `s`, `0`, and `1` are excluded to reduce visual and handwritten ambiguity.
- The pattern prevents long letter sequences and needs no escaping in URL paths, filenames, or HTML/CSS IDs.
- In default mode, typing needs no Shift key, `_`, `-`, or other punctuation.

Set `allow_uppercase` to `true` to sample letter positions from the combined
uppercase and lowercase alphabet.

## API

| API | Description |
| --- | --- |
| `tidyid(length, allow_uppercase)` | Generate a 3–256 character ID. |
| `is_valid_id(value, length, allow_uppercase)` | Check format and an optional exact length. |
| `ensure_valid_id(value, length, allow_uppercase)` | Return `InvalidIdLengthError` or `InvalidIdFormatError` through `ValidationError`. |
| `get_id_capacity(length, allow_uppercase)` | Return the exact ID space as `BigUint` (`metrics` feature). |
| `get_id_entropy(length, allow_uppercase)` | Return entropy in bits (`metrics` feature). |

These functions are thread-safe.

Constants: `LETTERS`, `LETTERS_WITH_UPPERCASE`, `DIGITS`, `DEFAULT_LENGTH`, `MIN_LENGTH`, `MAX_LENGTH`.

Errors: `InvalidIdLengthError`, `InvalidIdFormatError`, `GenerateError`, `ValidationError`.

## Performance

`cargo bench --bench throughput` measures complete public `tidyid` calls,
including secure random sampling and `String` allocation.

## Security

- **Unpredictability** Native targets use the operating system CSPRNG through [`getrandom`](https://docs.rs/getrandom). Browser-oriented `wasm32-unknown-unknown` builds use Web Crypto when the `wasm_js` feature is enabled. TidyID never uses a predictable PRNG.
- **Uniformity** Letter positions use rejection sampling, while digit positions use an exact eight-way mapping. Both avoid modulo bias, so every valid ID of the same length and mode has equal probability.

  <img src="https://raw.githubusercontent.com/sheldonix/tidyid-rust/main/docs/media/uniformity-default.svg" alt="Observed Rust TidyID default-mode letter and digit frequencies remain close to their expected uniform probabilities" width="680">

  *Default mode (`allow_uppercase = false`): observed frequencies from 10,000,000 generated 3-character IDs stay close to their expected uniform distribution.*

  <img src="https://raw.githubusercontent.com/sheldonix/tidyid-rust/main/docs/media/uniformity-allow-uppercase.svg" alt="Observed Rust TidyID uppercase-enabled character frequencies remain close to their expected uniform probabilities" width="680">

  *Uppercase-enabled mode (`allow_uppercase = true`): observed letter and digit frequencies from 10,000,000 generated 3-character IDs stay close to their expected uniform distribution.*

- **Collision-aware** Choose a length for your scale to make collisions extremely unlikely. Use a database `PRIMARY KEY` or `UNIQUE` constraint when absolute uniqueness must be enforced.

<!-- BEGIN GENERATED CAPACITY TABLES -->
  > **Default mode (`allow_uppercase = false`)**
  >
  > | Length | Capacity | Entropy |
  > | ---: | ---: | ---: |
  > | 8 | 7,256,313,856 | 32.76 bits |
  > | 10 | 1,277,111,238,656 | 40.22 bits |
  > | 12 | 224,771,578,003,456 | 47.68 bits |
  > | 16 | 19,146,942,100,646,395,904 | 64.05 bits |
  > | 23 | 6,315,282,784,770,463,143,393,492,992 | 92.35 bits |
  > | 32 | 366,605,391,805,505,419,895,548,144,464,707,977,216 | 128.11 bits |

  > **Uppercase allowed (`allow_uppercase = true`)**
  >
  > | Length | Capacity | Entropy |
  > | ---: | ---: | ---: |
  > | 8 | 464,404,086,784 | 38.76 bits |
  > | 10 | 163,470,238,547,968 | 47.22 bits |
  > | 12 | 57,541,523,968,884,736 | 55.68 bits |
  > | 16 | 39,212,937,422,123,818,811,392 | 75.05 bits |
  > | 23 | 413,878,372,582,717,072,565,435,956,723,712 | 108.35 bits |
  > | 32 | 1,537,654,461,271,398,604,689,577,164,520,902,527,668,977,664 | 150.11 bits |
<!-- END GENERATED CAPACITY TABLES -->

Use 16 or more characters for large public datasets. For security tokens, choose the length based on your threat model. A 32-character TidyID provides 128.11 bits of entropy by default, or 150.11 bits with `allow_uppercase = true`.

## Database uniqueness

Use a primary key or unique constraint. Insert first and retry only an ID conflict—never query before inserting:

```rust,no_run
use sqlx::PgPool;
use tidyid::tidyid;

async fn create_resource(pool: &PgPool) -> Result<String, Box<dyn std::error::Error>> {
    for _ in 0..128 {
        let id = tidyid(16, false)?;
        let inserted = sqlx::query_scalar::<_, String>(
            r#"INSERT INTO resources (id) VALUES ($1)
               ON CONFLICT (id) DO NOTHING RETURNING id"#,
        )
        .bind(&id)
        .fetch_optional(pool)
        .await?;

        if inserted.is_some() {
            return Ok(id);
        }
    }
    Err("unable to insert a resource with a unique TidyID".into())
}
```

Propagate network, permission, transaction, and non-ID constraint errors.

## Requirements

- Rust `1.85` or newer
- A target supported by `getrandom`
- For browser-oriented `wasm32-unknown-unknown`, enable `wasm_js`:

  ```toml
  [dependencies]
  tidyid = { version = "2.1.1", features = ["wasm_js"] }
  ```

## Development

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo bench
cargo run --release --features metrics --example generate_readme_data
```

## License

MIT
