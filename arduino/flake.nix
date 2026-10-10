{
  description = "Monitor Build Environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
      ...
    }:
    let
      supportedSystems = [
        "aarch64-linux"
        "x86_64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      forEachSupportedSystem =
        f:
        nixpkgs.lib.genAttrs supportedSystems (
          system:
          f {
            pkgs = import nixpkgs {
              inherit system;
              overlays = [ self.overlays.default ];
            };
          }
        );
    in
    {
      # Overlay for the Rust toolchain
      overlays.default = final: prev: {
        rustToolchain = fenix.packages.${prev.stdenv.hostPlatform.system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-DnyK5MS+xYySA+csnnMogu2gtEfyiy10W0ATmAvmjGg=";
        };
      };

      # Inputs for shell environments
      devShells = forEachSupportedSystem (
        { pkgs }: {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              rustToolchain
              rustc
              cargo-generate
              bacon
              cargo-shear
              pkgsCross.avr.buildPackages.gcc
              pkg-config
              udev
              avrdude
              ravedude
            ];
            env = {
              # Required by rust-analyzer
              RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
              LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
                # load external libraries that you need in your rust project here
                pkgs.udev
              ];
            };
          };
        }
      );
    };
}
