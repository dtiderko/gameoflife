{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain (
          p: p.rust-bin.nightly.latest.default.override {
            targets = [
              "x86_64-unknown-linux-gnu"
              "wasm32-unknown-unknown"
            ];
          }
        );

        crate = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;
          buildInputs = with pkgs; [
            alsa-lib # audio
            # Other dependencies
            libudev-zero
            pkg-config
            libxkbcommon
            wayland
            mold
            clang
          ];
        };
      in
      {
        packages.default = crate;
        devShells.default = craneLib.devShell {
          inputsFrom = [ crate ];
          packages = with pkgs; [
            wasm-bindgen-cli_0_2_106
            http-server
          ];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs;[
            vulkan-loader
            libxkbcommon
          ]);
        };
      }
    );
}
