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

# Run tests.
test *FLAGS:
    cargo nextest run --all-features --workspace {{FLAGS}}

# Runs tests with a coverage report.
test-cov *FLAGS:
    # cargo llvm-cov nextest --all-features --workspace --fail-under-lines 85 {{FLAGS}}
    cargo llvm-cov nextest --all-features --workspace {{FLAGS}}

# Runs doc tests.
doctest:
    cargo test --workspace --doc

# Runs the complete counter functional-coverage example.
functional-coverage-example OUTPUT='target/vvm-functional-coverage':
    #!/usr/bin/env bash
    set -euo pipefail

    rm -rf "{{OUTPUT}}"
    cargo run -p cargo-vvm -- coverage --output "{{OUTPUT}}" --name counter -- test -p vvm-example-counter counter_

# Runs in-scope Rust coverage with threshold enforcement and generates a cobertura report for gitlab.
test-cov-ci *FLAGS:
    mkdir -p coverage
    @just test-cov --cobertura --output-path coverage/cobertura.xml
    @just test-cov --summary-only | tee -a coverage/coverage.txt

# Build and run.
run *FLAGS:
    cargo run {{FLAGS}}

# Build release.
build *FLAGS:
    cargo build --workspace --release {{FLAGS}}

# Cleans rust build artifacts.
clean:
    cargo clean

# Generate documentation. Add '-- open' to open the docs in a web page.
docs *FLAGS:
    cargo doc --no-deps --all-features --document-private-items --workspace {{FLAGS}}

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
    @just doctest
    @just test-cov-ci

# Installs pre-commit hooks.
install-hooks:
    pre-commit install

# Installs the cargo-vvm Cargo subcommand.
install:
    cargo install --path crates/cargo-vvm --locked

# Install pre-commit hooks.
pre-commit-install:
    pre-commit install

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
    echo # things required by thorough-check
    cargo udeps -V || cargo binstall cargo-udeps --no-confirm
    cargo audit -V || cargo binstall cargo-audit --no-confirm
    echo # installing markdown-toc
    npm list -g markdown-toc || npm install -g markdown-toc
    echo # installing git hooks
    pre-commit --version || pip install pre-commit
    pre-commit install || echo "failed to install git hooks!" 1>&2
