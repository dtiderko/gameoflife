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
          p: p.rust-bin.nightly.latest.default
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
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs;[
            vulkan-loader
            libxkbcommon
          ]);
        };
      }
    );
}
