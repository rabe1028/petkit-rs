# petkit-rs

`petkit-rs` is an unofficial Rust workspace for building PETKIT API clients and request builders. The workspace is split so protocol modeling, typed payloads, and HTTP transport integration can evolve independently.

## Crate layout

- `petkit-types`: shared data types used across requests and responses. This crate is `no_std`-friendly.
- `petkit-protocol`: PETKIT request/response protocol types and request specifications. This crate is `no_std`-friendly.
- `petkit-client`: transport-aware client helpers, authentication flow helpers, and optional `reqwest` / `ureq` integrations.

## Design: Sans-IO request builders

The core design is Sans-IO:

- `petkit-protocol` builds typed `RequestSpec` values.
- `petkit-client` can execute those specs through an injected transport.
- You can use the built-in `reqwest` / `ureq` transports or provide your own transport implementation.

That split keeps protocol logic reusable without requiring a specific runtime, TLS stack, or HTTP client.

## Feature flags

The workspace keeps the protocol and type crates lightweight, while `petkit-client` exposes optional transport features.

| Feature | Purpose |
| --- | --- |
| `async` | Enables async client and transport support. |
| `action-adapter` | Enables the thin action-string parser for sidecar-style commands such as `feed`, `litterbox_clean`, and `purifier_power`. |
| `blocking` | Enables blocking client and transport support. |
| `reqwest-async` | Enables the async `reqwest` transport. |
| `reqwest-blocking` | Enables the blocking `reqwest` transport. |
| `ureq-blocking` | Enables the blocking `ureq` transport. |
| `reqwest-native` | Backwards-compatible umbrella feature that enables both `reqwest` transports. |
| `rustls-tls` | Backwards-compatible alias for the current `reqwest` transport wiring. |

`petkit-types` and `petkit-protocol` are the `no_std`-friendly crates in this workspace. If you only need request building or protocol modeling, you can depend on those crates without bringing in a transport stack. `petkit-client --no-default-features --features async` is also intended for host-provided transports, including `wasm32-wasip2` plugins that call out to a host HTTP capability instead of linking `reqwest` or `ureq`.

## Examples

The detail-read examples all share these env vars:
`PETKIT_EMAIL`, `PETKIT_PASSWORD`, `PETKIT_REGION`, `PETKIT_TIMEZONE_ID`,
`PETKIT_TIMEZONE_OFFSET`, and `PETKIT_BASE_URL`.

To force a specific discovered device, each family also accepts optional
`PETKIT_<FAMILY>_DEVICE_ID`, `PETKIT_<FAMILY>_DEVICE_TYPE`, and
`PETKIT_<FAMILY>_DEVICE_NAME` overrides.

| Style | Device | Run command |
| --- | --- | --- |
| `reqwest` async | feeder | `cargo run -p petkit-client --example reqwest_async_feeder_detail --no-default-features --features async,reqwest-async` |
| `reqwest` async | litter | `cargo run -p petkit-client --example reqwest_async_litter_detail --no-default-features --features async,reqwest-async` |
| `reqwest` async | purifier | `cargo run -p petkit-client --example reqwest_async_purifier_detail --no-default-features --features async,reqwest-async` |
| `reqwest` blocking | feeder | `cargo run -p petkit-client --example reqwest_blocking_feeder_detail --no-default-features --features blocking,reqwest-blocking` |
| `reqwest` blocking | litter | `cargo run -p petkit-client --example reqwest_blocking_litter_detail --no-default-features --features blocking,reqwest-blocking` |
| `reqwest` blocking | purifier | `cargo run -p petkit-client --example reqwest_blocking_purifier_detail --no-default-features --features blocking,reqwest-blocking` |
| `ureq` blocking | feeder | `cargo run -p petkit-client --example ureq_blocking_feeder_detail --no-default-features --features blocking,ureq-blocking` |
| `ureq` blocking | litter | `cargo run -p petkit-client --example ureq_blocking_litter_detail --no-default-features --features blocking,ureq-blocking` |
| `ureq` blocking | purifier | `cargo run -p petkit-client --example ureq_blocking_purifier_detail --no-default-features --features blocking,ureq-blocking` |
| host callback async | canned transport | `cargo run -p petkit-client --example host_callback_async --no-default-features --features async` |

Each example logs in, loads `family_list`, picks a feeder/litter/purifier from
discovery or env vars, builds a concrete `RequestSpec`, reads the broad
`device_detail`/`deviceData` payload, and prints selected `settings.*` and
`state.*` keys without mutating the device.

## Minimal async example

```rust
use petkit_client::ReqwestAsyncPetkitClient;
use petkit_protocol::BaseUrl;
use petkit_types::{ClientContext, ClientProfile};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let context = ClientContext::new(ClientProfile::default(), "UTC", "0");
    let client = ReqwestAsyncPetkitClient::new_reqwest(
        context,
        BaseUrl::Regional("https://api.petkt.com/latest/".into()),
    );

    let families = client.family_list().await?;
    println!("{}", families.len());
    Ok(())
}
```

## Minimal blocking example

```rust
use petkit_client::ReqwestBlockingPetkitClient;
use petkit_protocol::BaseUrl;
use petkit_types::{ClientContext, ClientProfile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let context = ClientContext::new(ClientProfile::default(), "UTC", "0");
    let client = ReqwestBlockingPetkitClient::new_reqwest(
        context,
        BaseUrl::Regional("https://api.petkt.com/latest/".into()),
    );

    let families = client.family_list()?;
    println!("{}", families.len());
    Ok(())
}
```

## Device operation pattern

The usual flow is:

```rust
use petkit_types::{DeviceId, FeederDeviceDetailResponse, FeederDeviceType};

let device_id = DeviceId::new(42)?;
let feeder = client
    .authenticated()
    .feeder(FeederDeviceType::D4s, device_id);
let request = feeder.device_detail_request();
println!("request path: {}", request.path);

let response: FeederDeviceDetailResponse = feeder.device_detail().await?;
println!("{:?}", response.settings_member("lightMode")?);
```

From there you have two common options:

1. Use the client-backed scopes (`client.authenticated().feeder(...).device_detail()`) for JSON envelope endpoints that decode into typed results.
2. For raw transport control, borrow the request builders through `authenticated_protocol()` or `scope.requests()`, send the `RequestSpec` through the transport directly, then parse with `petkit_protocol::parse_api_response(...)`. Async raw sends need `AsyncTransport` in scope, and blocking raw sends need `BlockingTransport` in scope when calling `client.transport().send(...)`.

This keeps device-specific operations composable while still letting you inject your own networking layer.

## Host callback / Wasm transport

For Wasm plugins or other capability-based hosts, use `HostCallbackTransport` to turn a host function into the `RequestSpec -> ResponseParts` transport that `AsyncPetkitClient` expects:

```rust
use petkit_client::host_callback::HostCallbackTransport;
use petkit_client::AsyncPetkitClient;
use petkit_protocol::{BaseUrl, RequestSpec, ResponseParts};
use petkit_types::{ClientContext, ClientProfile};

async fn host_send(request: RequestSpec) -> Result<ResponseParts, HostError> {
    // Forward method/url/headers/query/form_fields to the host and return its response parts.
    todo!()
}

let transport = HostCallbackTransport::from_fn(host_send);
let client = AsyncPetkitClient::with_session(
    ClientContext::new(ClientProfile::default(), "UTC", "0"),
    BaseUrl::Regional("https://api.petkt.com/latest/".into()),
    "session-id",
    transport,
);
```

Camera-capable feeder/litter scopes now expose typed `start_live()` responses with `channel_id`, `rtc_token`, `rtm_token`, `uid`, `app_rtm_user_id`, and `dev_rtm_user_id`. The Agora/WHEP bridge itself remains outside core; keep that in a sidecar or a dedicated integration crate.

`family_list()` results can be flattened with `flatten_devices` or wrapped in `DeviceCatalog` for numeric, unique, or opaque (`"<device_type>:<device_id>"`) id resolution. `client.authenticated().device_detail_for(&summary)` follows the discovered family/type to the correct typed `device_detail` endpoint.

`IotConfigSet::aliyun_mqtt_connection_summary(...)` builds PetKit/Aliyun MQTT connection data (`client_id`, `username`, HMAC-SHA256 password, and `/user/get`/`/user/update` topics). `petkit-protocol` also exposes raw BLE frame helpers plus a small `BleGattWriter` trait so platform-specific GATT scan/write implementations can stay outside the core crates.

## Quality commands

Install pinned local tools once with [mise](https://mise.jdx.dev/):

```bash
mise install
```

```bash
make quality
make ci-quality
make doc
cargo test
make test-no-std
make feature-matrix
make shellcheck
make shfmt-check
make actionlint
make msrv
make minimal-versions
make deny
make machete
make typos
make fuzz-check
make actions-local-quality
make actions-local-msrv
make actions-local-minimal-versions
```

If your local workflow runs the tools directly, the workspace also checks cleanly with commands such as `cargo test`, `cargo deny`, `cargo machete`, `typos`, `shellcheck`, `shfmt`, and `actionlint`. Tool versions for cargo-installed binaries are pinned in `.mise.toml`. The CI quality workflow checks shell scripts, GitHub Actions workflows, stable/latest dependencies, the supported `petkit-client` feature matrix (including `--no-default-features`), the declared MSRV, and direct-minimal dependency resolution.

## Caveat

PETKIT API behavior can vary by device model, firmware version, account region, and upstream app changes. Treat request/response behavior as subject to change, especially for lesser-documented device operations.
