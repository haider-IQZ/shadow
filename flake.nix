{
  description = "Shadow package manager development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, rust-overlay, ... }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in {
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          # The overlay revision in flake.lock pins what "latest" means.
          rust = pkgs.rust-bin.nightly.latest.default.override {
            extensions = [ "rust-src" "rust-analyzer" "rustfmt" "clippy" ];
          };
        in {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rust
              lua5_4
              pkg-config
              clang
              cmake
              gnumake
              openssl
              zstd
              git
              curl
              gnutar
              gzip
              xz
              unzip
            ];

            # For crates which generate C bindings through bindgen.
            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
            RUST_BACKTRACE = "1";
          };
        });
    };
}
