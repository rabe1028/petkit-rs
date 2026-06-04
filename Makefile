.PHONY: fmt fmt-check clippy test test-no-std feature-matrix shellcheck shfmt shfmt-check actionlint doc quality deny machete typos msrv minimal-versions ci-quality ci-msrv ci-minimal-versions mutants-list mutants-check mutants-focus fuzz-check petkit-live-smoke actions-local-quality actions-local-msrv actions-local-minimal-versions actions-local-mutants actions-local-fuzz

ACTRUN := sh scripts/actrun.sh
MISE := mise exec --
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

feature-matrix:
	sh scripts/ci-feature-matrix.sh

shellcheck:
	$(MISE) shellcheck scripts/*.sh

shfmt:
	$(MISE) shfmt -w -i 2 scripts/*.sh

shfmt-check:
	$(MISE) shfmt -d -i 2 scripts/*.sh

actionlint:
	$(MISE) actionlint

doc:
	RUSTDOCFLAGS="-D rustdoc::broken-intra-doc-links" cargo doc --workspace --all-features --no-deps

quality: fmt-check clippy test test-no-std feature-matrix doc

deny:
	$(MISE) cargo deny check bans licenses sources

machete:
	$(MISE) cargo machete

typos:
	$(MISE) typos .

msrv:
	MSRV=$(MSRV) sh scripts/ci-msrv.sh

minimal-versions:
	MSRV=$(MSRV) NIGHTLY=$(NIGHTLY) sh scripts/ci-minimal-versions.sh

ci-quality: shellcheck shfmt-check actionlint quality deny machete typos

ci-msrv: msrv

ci-minimal-versions: minimal-versions

mutants-list:
	$(MISE) cargo mutants --workspace --all-features --list

mutants-check:
	$(MISE) cargo mutants --workspace --all-features --check --in-place --timeout 120 --cap-lints true --profile mutants

mutants-focus:
	sh scripts/mutation-smoke.sh

fuzz-check:
	cd fuzz && $(MISE) cargo fuzz check api_response
	cd fuzz && $(MISE) cargo fuzz check base_url_join
	cd fuzz && $(MISE) cargo fuzz check ble_frame

petkit-live-smoke:
	sh scripts/petkit-live-smoke.sh

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
