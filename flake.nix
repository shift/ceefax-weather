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
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
        };

      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "ceefax-weather";
          version = "0.2.0";

          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];

          # This hook ensures the 'templates' directory is copied into the final build output,
          # so the application can find the .toml files at runtime.
          postInstall = ''
            mkdir -p $out/bin
            cp -r templates $out/bin/
          '';

          meta = {
            description = "Ceefax-style weather map using wttr.in, written in Rust";
            license = pkgs.lib.licenses.mit;
          };
        };

        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${system}.default ];
          buildInputs = [ rustToolchain pkgs.rust-analyzer ];
        };
      });
}

