.PHONY: fmt fmt-check clippy test test-no-std doc quality deny machete typos msrv minimal-versions ci-quality ci-msrv ci-minimal-versions mutants-list mutants-check mutants-focus fuzz-check actions-local-quality actions-local-msrv actions-local-minimal-versions actions-local-mutants actions-local-fuzz

ACTRUN := sh scripts/actrun.sh
MSRV ?= 1.88.0
NIGHTLY ?= nightly

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test --workspace --all-targets --all-features

test-no-std:
	cargo check -p petkit-types --no-default-features
	cargo check -p petkit-protocol --no-default-features

doc:
	RUSTDOCFLAGS="-D rustdoc::broken-intra-doc-links" cargo doc --workspace --all-features --no-deps

quality: fmt-check clippy test test-no-std doc

deny:
	cargo deny check bans licenses sources

machete:
	cargo machete

typos:
	typos .

msrv:
	MSRV=$(MSRV) sh scripts/ci-msrv.sh

minimal-versions:
	MSRV=$(MSRV) NIGHTLY=$(NIGHTLY) sh scripts/ci-minimal-versions.sh

ci-quality: quality deny machete typos

ci-msrv: msrv

ci-minimal-versions: minimal-versions

mutants-list:
	cargo mutants --workspace --all-features --list

mutants-check:
	cargo mutants --workspace --all-features --check --in-place --timeout 120 --cap-lints true --profile mutants

mutants-focus:
	sh scripts/mutation-smoke.sh

fuzz-check:
	cd fuzz && cargo fuzz check api_response
	cd fuzz && cargo fuzz check base_url_join
	cd fuzz && cargo fuzz check ble_frame

actions-local-quality:
	$(ACTRUN) workflow run .github/workflows/quality.yml --job verify --trigger pull_request --local --include-dirty

actions-local-msrv:
	$(ACTRUN) workflow run .github/workflows/quality.yml --job msrv --trigger pull_request --local --include-dirty

actions-local-minimal-versions:
	$(ACTRUN) workflow run .github/workflows/quality.yml --job minimal-versions --trigger pull_request --local --include-dirty

actions-local-mutants:
	$(ACTRUN) workflow run .github/workflows/tooling.yml --job mutants --trigger push --local --include-dirty

actions-local-fuzz:
	$(ACTRUN) workflow run .github/workflows/tooling.yml --job fuzz --trigger push --local --include-dirty
