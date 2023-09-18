# openid-2.0

Based on [`node-openid`](https://github.com/havard/node-openid) and tries to be at least a tiny (🤏) bit spec compliant.

## Constraints

- Always using the `checkid_setup` mode (not using immediate mode)
- No extensions are implemented
- I _think_ the `strict` mode of `node-openid` corresponds to enforcing encryption which we do
  - The constructed `reqwest::Client` uses at least `TLSv1.2` and is set to HTTPS only.

## Steam Authetication

1) The user visits `/api/auth/steam/login`
2) A random nonce `N` is generated and saved
3) The user is redirected to Steam to log in
   1) Where steam redirects the user after a successful login is encoded in the URL
   2) Steam includes the nonce `N` when redirecting the user back to us
4) The user signs in to his Steam account and accepts the authentication request
5) The user is redirected to `/api/auth/steam/callback`
6) We check the validity of the info encoded in the URL parameters
7) We check that the nonce matches the nonce `N` and is not replayed
8) We send a request to Steam to validate the info encoded in the URL parameters
9) Steam approves the info (signature is correct)
10) The user successfully authenticated through Steam

## Notes

### Relevant Documentation

- [openid.net/specs/openid-authentication-2_0.html](https://openid.net/specs/openid-authentication-2_0.html) or [.txt](https://openid.net/specs/openid-authentication-2_0.txt)
- [docs.oasis-open.org/xri/2.0/specs/cd02/xri-resolution-V2.0-cd-02.html](http://docs.oasis-open.org/xri/2.0/specs/cd02/xri-resolution-V2.0-cd-02.html)
- [actix.rs/docs](https://actix.rs/docs)
- [serde.rs](https://serde.rs/)
- [docs.rs/tokio/latest/tokio/#feature-flags](https://docs.rs/tokio/latest/tokio/#feature-flags)

### To Do

- Better names for packages, e. g., `openid/xml` is not a good name at all
- Move some of the constants in `openid/constants` into an enum or something
- Implement full serde serializer and deserializer for key values
  - See `src/openid/util/comma_separated_serde.rs`
  - [openid.net/specs/openid-authentication-2_0.html#rfc.section.4.1.1](https://openid.net/specs/openid-authentication-2_0.html#rfc.section.4.1.1)
  - [github.com/nox/serde_urlencoded](https://github.com/nox/serde_urlencoded)

### Cool Stuff

- `actix_web` error handling
  - [dev.to/chaudharypraveen98/error-handling-in-actix-web-4mm](https://dev.to/chaudharypraveen98/error-handling-in-actix-web-4mm)
- `serde` `serialize_with` and `deserialize_with` examples
  - [gist.github.com/ripx80/33f80618bf13e3f4964b0d75c62bfd28](https://gist.github.com/ripx80/33f80618bf13e3f4964b0d75c62bfd28)
- Echoing HTTP requests
  - [stackoverflow.com/a/9770981/7988127](https://stackoverflow.com/a/9770981/7988127)
- Actor pattern with Tokio
  - [ryhl.io/blog/actors-with-tokio/](https://ryhl.io/blog/actors-with-tokio/)
- Don't unconditionally use `tokio::sync::Mutex`
  - [tokio.rs/tokio/tutorial/shared-state#on-using-stdsyncmutex](https://tokio.rs/tokio/tutorial/shared-state#on-using-stdsyncmutex)

## Credits 💖

- [actix-web](https://crates.io/crates/actix-web): Web framework.
- [anyhow](https://crates.io/crates/anyhow): Simplifies error handling.
- [base64](https://crates.io/crates/base64): Encode and decode data in Base64 format.
- [chrono](https://crates.io/crates/chrono): Date and time library.
- [chrono-humanize](https://crates.io/crates/chrono-humanize): Formats time in a human-readable way.
- [log](https://crates.io/crates/log): Logging facade.
- [rand](https://crates.io/crates/rand): Random number generation.
- [reqwest](https://crates.io/crates/reqwest): HTTP client for making web requests.
- [roxmltree](https://crates.io/crates/roxmltree): Fast and efficient XML library.
- [serde](https://crates.io/crates/serde): Serialization framework.
- [serde_json](https://crates.io/crates/serde_json): JSON serialization and deserialization using Serde.
- [serde_urlencoded](https://crates.io/crates/serde_urlencoded): URL encoding and decoding using Serde.
- [simplelog](https://crates.io/crates/simplelog): Simple logging library.
- [tokio](https://crates.io/crates/tokio): Asynchronous runtime.

And all other in the `Cargo.toml`! Descriptions above are generated using [ChatGPT](https://chat.openai.com/)

## [forsen]

![forsenSmug](https://cdn.7tv.app/emote/60ae877c229664e866c27c51/3x.webp)

| [forsen] | [forsen] | [forsen] | [forsen] | [forsen] | [forsen] | [forsen] |
| -------- | -------- | -------- | -------- | -------- | -------- | -------- |
| [forsen] | [forsen] | [forsen] | [forsen] | [forsen] | [forsen] | [forsen] |
| [forsen] | [forsen] | [forsen] | [forsen] | [forsen] | [forsen] | [forsen] |
| [forsen] | [forsen] | [forsen] | [forsen] | [forsen] | [forsen] | [forsen] |
| [forsen] | [forsen] | [forsen] | [forsen] | [forsen] | [forsen] | [forsen] |
| [forsen] | [forsen] | [forsen] | [forsen] | [forsen] | [forsen] | [forsen] |
| [forsen] | [forsen] | [forsen] | [forsen] | [forsen] | [forsen] | [forsen] |
| [forsen] | [forsen] | [forsen] | [forsen] | [forsen] | [forsen] | [forsen] |
| [forsen] | [forsen] | [forsen] | [forsen] | [forsen] | [forsen] | [forsen] |

[forsen]: https://www.twitch.tv/forsen
