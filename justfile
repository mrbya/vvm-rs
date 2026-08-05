#!/usr/bin/env just --justfile
set dotenv-load := true

# Output this list.
list:
    @just --list

# Output this list.
help:
    @just list

# Apply strict formatting.
fmt *FLAGS:
    cargo +nightly fmt --all {{FLAGS}}

# Run clippy on codebase, tests and examples.
check *FLAGS:
    cargo clippy --tests --examples --all-targets --all-features --workspace {{FLAGS}}

# Run every existing test target. Prefer a category-specific recipe while developing.
test *FLAGS:
    cargo nextest run --all-features --workspace {{FLAGS}}

# Run private implementation tests only.
test-unit *FLAGS:
    cargo nextest run --all-features -p vvm-rs -p vvm-core -p vvm-build -p vvm-macros -p cargo-vvm --lib {{FLAGS}}

# Run public crate contracts that do not require generated native models.
test-integration *FLAGS:
    cargo nextest run --all-features -p vvm-rs --test public_api {{FLAGS}}
    cargo nextest run --all-features -p vvm-core --test properties {{FLAGS}}
    cargo nextest run --all-features -p cargo-vvm --test cli -E 'not test(coverage_command_writes_merged_and_rendered_counter_reports) and not test(coverage_command_preserves_test_failure_after_writing_counter_reports)' {{FLAGS}}

# Run procedural-macro compile-pass and compile-fail cases.
test-ui *FLAGS:
    cargo nextest run --all-features -p vvm-macros --test trybuild {{FLAGS}}

# Run clean pure-Rust consumer fixtures with isolated Cargo targets.
test-fixtures *FLAGS:
    cargo nextest run --all-features -p vvm-rs --test fixtures pure_consumer {{FLAGS}}

# Run tests that require Verilator and native C++ compilation.
test-native *FLAGS:
    cargo nextest run --all-features -p vvm-rs --test integration_tests {{FLAGS}}
    @just test-native-fixtures {{FLAGS}}
    @just test-examples {{FLAGS}}

# Run maintained user-facing native examples.
test-examples *FLAGS:
    cargo nextest run --all-features -p vvm-example-counter -p vvm-example-sync-fifo -p vvm-example-timed-uart -p vvm-example-async-fifo -p vvm-example-tri-state-bus {{FLAGS}}

# Run copied, isolated native HDL fixture workspaces.
test-native-fixtures *FLAGS:
    cargo nextest run --all-features -p vvm-rs --test fixtures -E 'test(native_)' {{FLAGS}}

# Run the release-facing native workflows.
test-e2e *FLAGS:
    @just test-examples {{FLAGS}}
    cargo nextest run --all-features -p vvm-rs --test fixtures -E 'test(clean_consumer_generates_and_executes_a_dut)' {{FLAGS}}
    cargo nextest run --all-features -p cargo-vvm --test cli -E 'test(coverage_command_writes_merged_and_rendered_counter_reports)' {{FLAGS}}
    cargo nextest run --all-features -p cargo-vvm --test cli -E 'test(coverage_command_preserves_test_failure_after_writing_counter_reports)' {{FLAGS}}
    @just functional-coverage-fifo
    @just functional-coverage-async-fifo

# Verify the publishable crate archives without contacting crates.io.
test-package *FLAGS:
    cargo nextest run --all-features -p vvm-rs --test fixtures packaged_consumer {{FLAGS}}

# Run all tests that do not need Verilator.
test-fast *FLAGS:
    @just test-unit {{FLAGS}}
    @just test-integration {{FLAGS}}
    @just test-ui {{FLAGS}}
    @just test-fixtures {{FLAGS}}

# Run every test category (equivalent to `test` but with category separation).
test-all *FLAGS:
    @just test-fast {{FLAGS}}
    @just test-native {{FLAGS}}
    @just test-e2e {{FLAGS}}
    @just test-package {{FLAGS}}

# Runs tests with a coverage report.
test-cov *FLAGS:
    cargo llvm-cov nextest --all-features --workspace --fail-under-lines 90 {{FLAGS}}

# Runs doc tests.
doctest:
    cargo test --workspace --doc

# Runs the complete counter functional-coverage example.
functional-coverage-example OUTPUT='target/vvm-functional-coverage':
    #!/usr/bin/env bash
    set -euo pipefail

    rm -rf "{{OUTPUT}}"
    cargo run -p cargo-vvm -- coverage --output "{{OUTPUT}}" --name counter -- test -p vvm-example-counter counter_

# Runs the synchronous FIFO functional-coverage workflow.
functional-coverage-fifo OUTPUT='target/vvm-sync-fifo-coverage':
    #!/usr/bin/env bash
    set -euo pipefail

    rm -rf "{{OUTPUT}}"
    cargo run -p cargo-vvm -- coverage --output "{{OUTPUT}}" --name sync-fifo -- test -p vvm-example-sync-fifo fifo_

# Runs the asynchronous FIFO functional-coverage workflow.
functional-coverage-async-fifo OUTPUT='target/vvm-async-fifo-coverage':
    #!/usr/bin/env bash
    set -euo pipefail

    rm -rf "{{OUTPUT}}"
    cargo run -p cargo-vvm -- coverage --output "{{OUTPUT}}" --name async-fifo -- test -p vvm-example-async-fifo async_fifo_random

# Runs the timed UART functional-coverage workflow.
functional-coverage-timed-uart OUTPUT='target/vvm-timed-uart-coverage':
    #!/usr/bin/env bash
    set -euo pipefail

    rm -rf "{{OUTPUT}}"
    cargo run -p cargo-vvm -- coverage --output "{{OUTPUT}}" --name timed-uart -- test -p vvm-example-timed-uart uart_random_frames

# Produces retained text, JSON, Cobertura, and HTML coverage artifacts.
test-cov-ci *FLAGS:
    mkdir -p coverage
    cargo llvm-cov nextest --all-features --workspace --no-report {{FLAGS}}
    cargo llvm-cov report --cobertura --output-path coverage/cobertura.xml {{FLAGS}}
    cargo llvm-cov report --json --output-path coverage/coverage.json {{FLAGS}}
    cargo llvm-cov report --text --output-path coverage/coverage.txt {{FLAGS}}
    cargo llvm-cov report --html --output-dir coverage/html --fail-under-lines 90 {{FLAGS}}
    cargo llvm-cov report --summary-only | tee coverage/coverage-summary.txt
    @just test-cov-crate vvm-rs 85 {{FLAGS}}
    @just test-cov-crate vvm-core 85 {{FLAGS}}
    @just test-cov-crate vvm-build 85 {{FLAGS}}
    @just test-cov-crate vvm-macros 85 {{FLAGS}}
    @just test-cov-crate cargo-vvm 80 {{FLAGS}}

# Renders and enforces one measured package line-coverage floor from the shared profile.
test-cov-crate CRATE FLOOR *FLAGS:
    mkdir -p coverage/{{CRATE}}
    cargo llvm-cov report -p {{CRATE}} --summary-only --fail-under-lines {{FLOOR}} {{FLAGS}} | tee coverage/{{CRATE}}/summary.txt
    cargo llvm-cov report -p {{CRATE}} --json --output-path coverage/{{CRATE}}/coverage.json {{FLAGS}}
    cargo llvm-cov report -p {{CRATE}} --cobertura --output-path coverage/{{CRATE}}/cobertura.xml {{FLAGS}}

# Runs the recorded deterministic mutation baseline for checked time arithmetic.
mutation *FLAGS:
    cargo mutants --package vvm-core --file crates/vvm-core/src/time.rs --timeout 60 {{FLAGS}}

# Build and run.
run *FLAGS:
    cargo run {{FLAGS}}

# Build release.
build *FLAGS:
    cargo build --workspace --release {{FLAGS}}

# Cleans rust build artifacts.
clean:
    cargo clean

# Build the mdBook guide only.
docs-book:
    mdbook build docs/book

# Build the public API reference without private items.
docs-api:
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features -p vvm-rs -p vvm-build -p vvm-core -p vvm-macros

# Build maintainer-only API documentation with private items.
docs-internal:
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features --document-private-items -p vvm-rs -p vvm-build -p vvm-core -p vvm-macros

# Run documentation examples and build checks.
docs-test:
    @just doctest
    @just docs-book

# Validate generated doc suite entry points and local documentation links.
docs-links:
    @just docs-suite
    bash scripts/check-book-api-links.sh
    test -s public/getting-started.html

# Build the complete documentation.
docs:
    bash scripts/doc-suite.sh

# Serve the assembled site locally.
docs-serve:
    @just docs
    http-server -i public

# Remove assembled documentation outputs.
docs-clean:
    rm -rf public target/book target/docs-api

# Run Criterion benchmark suite.
benchmark *FLAGS:
    cargo bench --benches --features full {{FLAGS}}

# Save a named benchmark baseline for trend comparisons.
benchmark-save-baseline NAME='local' *FLAGS:
    cargo bench --benches --features full -- --save-baseline {{NAME}} {{FLAGS}}

# Compare current benchmarks against a saved baseline.
benchmark-compare-baseline NAME='local' *FLAGS:
    cargo bench --benches --features full -- --baseline {{NAME}} {{FLAGS}}

# Run one benchmark target (`parse`, `analysis`, `scenarios`, `corpus`).
benchmark-target TARGET *FLAGS:
    cargo bench --bench {{TARGET}} --features full {{FLAGS}}

# Audits codebase for vulnerabilities.
audit *FLAGS:
    cargo audit {{FLAGS}}

# Checks for unused dependencies.
unused *FLAGS:
    cargo +nightly udeps --all-targets --workspace

# Check formatting and linter checks, check for unused dependencies and audits for vulnerabilities.
thorough-check:
    @just fmt --check
    @just check -- -D warnings
    @just unused
    @just audit

# Re-indexes readme index.
index:
    markdown-toc -i README.md

# Runs formatting, tests and checks necessary before a commit.
pre-commit:
    @just fmt
    @just thorough-check
    @just doctest
    @just test-cov

# Similar to `pre-commit` command, but is not interactive and doesn't modify the codebase. Suitable for automated CI pipelines.
ci:
    @just thorough-check
    @just test-all
    @just doctest
    @just test-cov-ci

# Installs pre-commit hooks.
install-hooks:
    pre-commit install

# Installs the cargo-vvm Cargo subcommand.
install:
    cargo install --path crates/cargo-vvm --locked

# Builds docker image for a gitlab CI runner.
docker-build:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "${GITLAB_IMAGE_REGISTRY}" ]; then
        exit 1 # GITLAB_IMAGE_REGISTRY variable has to be set
    fi
    IMAGE_TAG="$(git rev-parse --short HEAD)"
    IMAGE_BASE="${GITLAB_IMAGE_REGISTRY}"
    IMAGE="${IMAGE_BASE}:${IMAGE_TAG}"
    IMAGE_LATEST="${IMAGE_BASE}:latest"
    sudo docker buildx build -f "./Dockerfile" -t "${IMAGE}" --load \
        --label "org.opencontainers.image.revision=$(git rev-parse HEAD)" \
        --label "org.opencontainers.image.created=$(date)" \
        --label "org.opencontainers.image.version=${IMAGE_TAG}" \
        .
    sudo docker tag "${IMAGE}" "${IMAGE_LATEST}"
    sudo docker push "${IMAGE}"
    sudo docker push "${IMAGE_LATEST}"

# Build a checksum-verified native CI image for one Verilator release.
docker-build-verilator VERSION SHA256:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "${GITLAB_IMAGE_REGISTRY}" ]; then
        exit 1 # GITLAB_IMAGE_REGISTRY variable has to be set
    fi
    IMAGE="${GITLAB_IMAGE_REGISTRY}:verilator-{{VERSION}}"
    sudo docker buildx build -f "ci/Dockerfile.verilator" \
        --build-arg "VERILATOR_VERSION={{VERSION}}" \
        --build-arg "VERILATOR_SHA256={{SHA256}}" \
        --tag "${IMAGE}" \
        --load .
    sudo docker push "${IMAGE}"

# Initializes the project by installing all tools necessary. Should be run once before beginning of development.
init:
    echo # installing nightly channel
    rustup install nightly
    echo # installing cargo-binstall for faster setup time
    cargo binstall -V || cargo install cargo-binstall
    echo # things required by test recipes
    cargo nextest -V || cargo binstall cargo-nextest --no-confirm
    echo # things required by coverage recipes
    rustup component add llvm-tools-preview
    cargo binstall cargo-llvm-cov --no-confirm
    echo # things required by manual or scheduled mutation testing
    cargo mutants -V || cargo binstall cargo-mutants --no-confirm
    echo # things required by thorough-check
    cargo udeps -V || cargo binstall cargo-udeps --no-confirm
    cargo audit -V || cargo binstall cargo-audit --no-confirm
    echo # installing markdown-toc
    npm list -g markdown-toc || npm install -g markdown-toc
    echo # installing pinned documentation builder
    mdbook -V || cargo install mdbook
    http-server -V || cargo install http-server
    echo # installing git hooks
    pre-commit --version || pip install pre-commit
    pre-commit install || echo "failed to install git hooks!" 1>&2
