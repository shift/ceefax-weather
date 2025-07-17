{
  description = "A CEEFAX-style weather map in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        # Use the rust-overlay to get the latest stable Rust toolchain
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Define the Rust toolchain to use
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
        };

      in
      {
        # The `nix build` command will produce this package
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "ceefax-weather";
          version = "0.1.0";

          src = ./.;

          # This file is crucial for reproducible builds in Rust projects with Nix
          cargoLock.lockFile = ./Cargo.lock;

          # Dependencies needed at build time
          nativeBuildInputs = [ pkgs.pkg-config ];
          
          # Dependencies needed by the Rust crates
          buildInputs = [ pkgs.openssl ];

          meta = {
            description = "Ceefax-style weather map using wttr.in, written in Rust";
            license = pkgs.lib.licenses.mit;
          };
        };

        # The `nix run` command will execute this app
        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
        };

        # A development shell can be entered with `nix develop`
        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${system}.default ];
          buildInputs = [ rustToolchain pkgs.rust-analyzer ];
        };
      });
}

