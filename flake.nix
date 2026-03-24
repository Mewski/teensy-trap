{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { flake-parts, ... } @ inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" ];

      perSystem = { pkgs, system, ... }:
        let
          rust-toolchain = inputs.fenix.packages.${system}.combine [
            inputs.fenix.packages.${system}.stable.cargo
            inputs.fenix.packages.${system}.stable.clippy
            inputs.fenix.packages.${system}.stable.rustc
            inputs.fenix.packages.${system}.stable.rustfmt
            inputs.fenix.packages.${system}.stable.rust-src
            inputs.fenix.packages.${system}.stable.llvm-tools
            inputs.fenix.packages.${system}.targets.thumbv7em-none-eabihf.stable.rust-std
          ];
          python = pkgs.python3.withPackages (ps: [ ps.pyserial ]);
        in
        {
          devShells.default = pkgs.mkShell {
            nativeBuildInputs = [
              rust-toolchain
              inputs.fenix.packages.${system}.rust-analyzer
              pkgs.gcc-arm-embedded
              pkgs.teensy-loader-cli
              pkgs.probe-rs-tools
              pkgs.flip-link
              pkgs.cargo-binutils
              python
            ];
          };
        };
    };
}
