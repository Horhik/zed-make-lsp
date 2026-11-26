# Make LSP for Zed

[make-ls](https://github.com/owenrumney/make-ls) language server for Makefiles in
[Zed](https://zed.dev): hover, completion, go-to-definition, find references,
document symbols, diagnostics, `.PHONY` code actions and formatting.

The extension registers the `Make` language (tree-sitter-make grammar,
highlighting, outline, runnables, indentation) and starts the `make-ls` server.

## Server resolution

1. `make-ls` found on `$PATH` (e.g. `go install github.com/owenrumney/make-ls/cmd/make-ls@latest`).
2. Cached download of make-ls `v0.1.25` from GitHub releases, stored under the
   extension work directory (`.local/share/zed/extensions/work/make-ls`).
3. `"lsp": { "make-ls": { "binary": { "path": "/absolute/path/to/make-ls" } } }`
   in Zed settings always takes precedence.

Prebuilt binaries exist for Linux/macOS x64 and arm64. On other platforms install
`make-ls` with Go and put it on `PATH`.

## Ubuntu

Prebuilt `extension.wasm` and `grammars/make.wasm` are committed, so no Rust or Go
toolchain is needed. The server binary is downloaded on first use.

Requirements:

- Zed 1.3.6 or newer (deb, snap, or flatpak)
- `git`, network access to `github.com` (server download)

Install without a toolchain:

```sh
git clone https://github.com/Horhik/zed-make-lsp.git
ln -sfn "$PWD/zed-make-lsp" ~/.local/share/zed/extensions/installed/make-ls
```

Restart Zed, open a `Makefile`, approve workspace trust. The extension index is
rebuilt on startup; `zed: extensions` should list `Make LSP` as a dev extension.

For flatpak, the data dir is
`~/.var/app/dev.zed.Zed/data/zed/extensions/installed` instead.

Rebuilding the artifacts on Ubuntu needs:

```sh
sudo apt install build-essential
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install 1.95.0 --profile minimal
rustup target add wasm32-wasip2 --toolchain 1.95.0
```

Zed downloads wasi-sdk automatically when it compiles the grammar.

## Install (dev extension)

Built artifacts are required for the extension to load:

```sh
# extension.wasm (wasm32-wasip2, Rust 1.95)
cargo build --release --target wasm32-wasip2
cp target/wasm32-wasip2/release/make_ls.wasm extension.wasm

# grammars/make.wasm (tree-sitter parser)
git clone https://github.com/tree-sitter-grammars/tree-sitter-make grammars/make
git -C grammars/make checkout 5e9e8f8ff3387b0edcaa90f46ddf3629f4cfeb1d
~/.local/share/zed/extensions/build/wasi-sdk/bin/clang \
  -fPIC -shared -Os -Wl,--export=tree_sitter_make -Wl,--max-memory=4294967296 \
  -o grammars/make.wasm -I grammars/make/src grammars/make/src/parser.c
```

Then install the directory as a dev extension in Zed:
`zed: extensions` → `Install Dev Extension` → select this folder.

Or link it manually (Zed detects dev extensions by a symlink in the installed
extensions directory):

```sh
ln -sfn "$PWD" ~/.local/share/zed/extensions/installed/make-ls
```

Zed rebuilds its extension index on startup.

## Notes

- Rust toolchain is pinned to `1.95.0` because newer `wasm32-wasip2` std
  imports a newer `wasi:cli` than Zed 1.3.6 implements.
- Do not install the registry `Make` extension alongside this one: both register
  the `Make` language.
- On NixOS the wasi-sdk `clang` needs `libtinfo`/`libstdc++` on
  `LD_LIBRARY_PATH`; `~/.local/share/zed/extensions/build/wasi-sdk/bin/clang`
  is wrapped accordingly.

## Credits

- [owenrumney/make-ls](https://github.com/owenrumney/make-ls) — language server (MIT)
- [caius/zed-make](https://github.com/caius/zed-make) — Make language queries (MIT)
- [tree-sitter-grammars/tree-sitter-make](https://github.com/tree-sitter-grammars/tree-sitter-make) — grammar (MIT)
