{
  description = "lxd-snapper: LXD snapshots, automated";

  inputs = {
    gitignore = {
      url = "github:hercules-ci/gitignore";
      flake = false;
    };

    naersk = {
      url = "github:nmattia/naersk";

      inputs = {
        nixpkgs = {
          follows = "nixpkgs";
        };
      };
    };

    nixpkgs = {
      url = "github:nixos/nixpkgs/nixos-20.09";
    };

    nixpkgs-mozilla = {
      url = "github:mozilla/nixpkgs-mozilla";
      flake = false;
    };
  };

  outputs = { self, gitignore, naersk, nixpkgs, nixpkgs-mozilla }:
    let
      pkgs = (import nixpkgs) {
        system = "x86_64-linux";

        overlays = [
          (import "${nixpkgs-mozilla}")
        ];
      };

      rust-pkg = (pkgs.rustChannelOf {
        rustToolchain = ./rust-toolchain;
        sha256 = "GpVvKLc8e4l5URj7YsJfgm2OwsNw35zhpGD/9Jzdt2o=";
      }).rust;

      gitignoreSource = (pkgs.callPackage gitignore { }).gitignoreSource;

      buildPackage = (pkgs.callPackage naersk {
        cargo = rust-pkg;
        rustc = rust-pkg;
      }).buildPackage;

    in
    {
      defaultPackage = {
        x86_64-linux = buildPackage {
          src = gitignoreSource ./.;
          doCheck = true;
          cargoTestOptions = args: args ++ [ "--all" ];
        };
      };
    };
}
