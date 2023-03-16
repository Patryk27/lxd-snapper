# These are lxd-snapper's acceptance tests; you can run them using:
#
# ```
# nix flake check
# ```

{ nixpkgs, lxd-snapper }:

let
  inherit (pkgs) lib;

  pkgs = import nixpkgs {
    system = "x86_64-linux";
  };

  lxdImage = import "${nixpkgs}/nixos/release.nix" {
    configuration = {
      documentation = {
        enable = lib.mkForce false;
      };

      environment = {
        noXlibs = lib.mkForce true;
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
          prelude = import ./tests/prelude.py.nix {
            inherit testPath;

            lxdConfig = ./tests/_fixtures/lxd-config.yaml;
            lxdImageMetadata = lxdImage.lxdMeta.${pkgs.system};
            lxdImageRootfs = lxdImage.lxdImage.${pkgs.system};
          };

        in
        prelude
        + "\n\n"
        + (builtins.readFile "${testPath}/test.py");

    in
    import "${testPath}/test.nix" {
      fw = rec {
        mkNode = config @ { ... }:
          lib.mkMerge [
            {
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
                hostId = "01234567";
              };

              virtualisation = {
                cores = 2;
                memorySize = 2048;
                diskSize = 2048;

                lxd = {
                  enable = true;
                  package = testPkgs.lxd;
                };

                qemu = {
                  options = [
                    "-rtc base=2018-01-01T12:00:00"
                  ];
                };
              };
            }

            config
          ];

        mkTest = { nodes }:
          pkgs.nixosTest {
            inherit testScript nodes;

            name = testName;
          };

        mkDefaultTest = mkTest {
          nodes = {
            machine = mkNode { };
          };
        };
      };
    };

  mkTests = { tests, lxds }:
    let
      testCombinations =
        lib.cartesianProductOfSets {
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
    ./tests/backup-and-prune
    ./tests/backup-and-prune-with-projects
    ./tests/backup-and-prune-with-remotes
    ./tests/dry-run
    ./tests/hooks
    ./tests/timeout
  ];

  lxds = [
    { version = "4.0"; nixpkgs = "2d9888f61c80f28b09d64f5e39d0ba02e3923057"; }
    { version = "4.24"; nixpkgs = "d1c3fea7ecbed758168787fe4e4a3157e52bc808"; }
    { version = "5.1"; nixpkgs = "bf972dc380f36a3bf83db052380e55f0eaa7dcb6"; }
    { version = "5.5"; nixpkgs = "ee01de29d2f58d56b1be4ae24c24bd91c5380cea"; }
  ];
}
