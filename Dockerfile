FROM archlinux:latest

RUN pacman -Syyuu gcc git llvm clang rustup npm pkgconf cmake ninja make verilator --noconfirm --needed && \
    rustup default stable && \
    rustup update && \
    touch /root/.bashrc && \
    cargo install just && \
    pacman -Scc && \
    echo "export PATH=\"$PATH:$HOME/.cargo/bin\"" | tee -a "$HOME/.bashrc"

ENV PATH=/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

RUN rustup install nightly && \
    rustup target add x86_64-pc-windows-gnu && \
    cargo install cargo-audit --locked --features=fix && \
    cargo install cargo-binstall && \
    cargo install --locked tree-sitter-cli && \
    cargo binstall cargo-nextest --no-confirm && \
    rustup component add llvm-tools-preview && \
    cargo binstall cargo-llvm-cov --no-confirm && \
    cargo binstall cargo-criterion --no-confirm && \
    cargo binstall cargo-udeps --no-confirm && \
    cargo binstall mdbook --no-confirm && \
    npm install -g markdown-toc

CMD ["bash"]
