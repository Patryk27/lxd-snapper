# These are lxd-snapper's acceptance tests; you can run them using:
#
# ```
# nix flake check -j4
# ```

{ nixpkgs
, nixpkgs--lxd-4
, nixpkgs--lxd-5
, nixpkgs--lxd-6
, lxd-snapper
}:

let
  inherit (pkgs) lib;

  pkgs = import nixpkgs {
    system = "x86_64-linux";
  };

  lxdContainer = import "${nixpkgs}/nixos/release.nix" {
    configuration = {
      documentation = {
        enable = lib.mkForce false;
      };

      environment = {
        noXlibs = lib.mkForce true;
      };
    };
  };

  mkTest = testPath: testName: testNixpkgs:
    let
      testPkgs = import testNixpkgs {
        system = "x86_64-linux";
      };

      testScript =
        let
          prelude = import ./tests/prelude.py.nix {
            inherit testPath;

            lxdConfig = ./tests/_fixtures/lxd-config.yaml;
            lxdContainerMeta = lxdContainer.lxdContainerMeta.${pkgs.system};
            lxdContainerImage = lxdContainer.lxdContainerImage.${pkgs.system};
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
        lib.cartesianProduct {
          testPath = tests;
          testLxd = lxds;
        };

      mkTestFromCombination = { testPath, testLxd }:
        let
          testName = "${builtins.baseNameOf testPath}.lxd-${testLxd.version}";

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
    { version = "4"; nixpkgs = nixpkgs--lxd-4; }
    { version = "5"; nixpkgs = nixpkgs--lxd-5; }
    { version = "6"; nixpkgs = nixpkgs--lxd-6; }
  ];
}
