{ nixpkgs, lxd-snapper }:

let
  pkgs = import nixpkgs {
    system = "x86_64-linux";
  };

  lxd-image = import "${nixpkgs}/nixos/release.nix" {
    configuration = {
      documentation = {
        enable = pkgs.lib.mkForce false;
      };

      system = {
        stateVersion = "22.11";
      };
    };
  };

  mkTest = testPath: testName: testNixpkgsRev:
    let
      testNixpkgs = builtins.fetchGit {
        url = "https://github.com/NixOS/nixpkgs";
        rev = testNixpkgsRev;
        shallow = true;
      };

      testPkgs = import testNixpkgs {
        system = "x86_64-linux";
      };

      testScript =
        let
          common = import ./cases/common.nix {
            inherit testPath;

            lxd-config = ./fixtures/lxd-config.yaml;
            lxd-image-metadata = lxd-image.lxdMeta.${pkgs.system};
            lxd-image-rootfs = lxd-image.lxdImage.${pkgs.system};
          };

        in
        common
        + "\n\n"
        + (builtins.readFile "${testPath}/test.py");

    in
    pkgs.nixosTest {
      inherit testScript;

      name = testName;

      nodes = {
        machine = { ... }: {
          boot = {
            supportedFilesystems = [ "zfs" ];
          };

          environment = {
            systemPackages = with pkgs; [
              jq
              lxd-snapper
            ];
          };

          networking = {
            # Required for ZFS; value doesn't matter
            hostId = "01234567";
          };

          virtualisation = {
            cores = 2;
            memorySize = 2048;
            diskSize = 8192;

            lxd = {
              enable = true;
              package = testPkgs.lxd;
            };
          };
        };
      };
    };

  mkTests = { tests, lxds }:
    let
      testCombinations =
        pkgs.lib.cartesianProductOfSets {
          testPath = tests;
          testLxd = lxds;
        };

      mkTestFromCombination = { testPath, testLxd }:
        let
          testName = "${builtins.baseNameOf testPath}--${testLxd.version}";

        in
        {
          name = testName;
          value = mkTest testPath testName testLxd.nixpkgs;
        };

    in
    builtins.listToAttrs
      (builtins.map
        mkTestFromCombination
        testCombinations);

in
mkTests {
  tests = [
    ./cases/backup-and-prune
    ./cases/backup-and-prune-with-projects
    ./cases/dry-run
    ./cases/hooks
  ];

  lxds = [
    { version = "4.0.0"; nixpkgs = "2d9888f61c80f28b09d64f5e39d0ba02e3923057"; }
    { version = "4.10"; nixpkgs = "bed08131cd29a85f19716d9351940bdc34834492"; }
    { version = "4.24"; nixpkgs = "d1c3fea7ecbed758168787fe4e4a3157e52bc808"; }
    { version = "5.1"; nixpkgs = "bf972dc380f36a3bf83db052380e55f0eaa7dcb6"; }
    { version = "5.5"; nixpkgs = "ee01de29d2f58d56b1be4ae24c24bd91c5380cea"; }
  ];
}
