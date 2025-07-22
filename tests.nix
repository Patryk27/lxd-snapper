# These are lxd-snapper's acceptance tests; you can run them using:
#
# ```
# nix flake check -j4
# ```

{
  nixpkgs,
  lxd-snapper,
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
    };
  };

  mkTest =
    testPath: testName: testFlavor:
    let
      testScript =
        let
          prelude = import ./tests/prelude.py.nix {
            inherit testPath testFlavor;

            lxdConfig = ./tests/_fixtures/lxd-config.yaml;
            lxdContainerMeta = lxdContainer.lxdContainerMeta.${pkgs.system};
            lxdContainerImage = lxdContainer.lxdContainerImage.${pkgs.system};
          };

        in
        prelude + "\n\n" + (builtins.readFile "${testPath}/test.py");

    in
    import "${testPath}/test.nix" {
      fw = rec {
        mkNode =
          config@{ ... }:
          lib.mkMerge [
            {
              boot = {
                supportedFilesystems = [ "zfs" ];
              };

              environment = {
                systemPackages =
                  let
                    lxc-or-incus = pkgs.writeShellScriptBin "lxc-or-incus" ''
                      ${if testFlavor == "lxd" then "lxc" else "incus"} $@
                    '';

                    lxd-or-incus = pkgs.writeShellScriptBin "lxd-or-incus" ''
                      ${if testFlavor == "lxd" then "lxd" else "incus admin"} $@
                    '';

                  in
                  with pkgs;
                  [
                    jq
                    lxc-or-incus
                    lxd-or-incus
                    lxd-snapper
                  ];
              };

              networking = {
                hostId = "01234567";

                nftables = {
                  enable = testFlavor == "incus";
                };
              };

              virtualisation = {
                cores = 2;
                memorySize = 2048;
                diskSize = 2048;

                incus = {
                  enable = testFlavor == "incus";
                };

                lxd = {
                  enable = testFlavor == "lxd";
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

        mkTest =
          { nodes }:
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

  mkTests' =
    tests: testFlavor:
    builtins.listToAttrs (
      builtins.map (
        testPath:
        let
          testName = builtins.baseNameOf testPath;

        in
        {
          name = "${testName}.${testFlavor}";
          value = mkTest testPath testName testFlavor;
        }
      ) tests
    );

  mkTests =
    tests:
    let
      lxdTests = mkTests' tests "lxd";
      incusTests = mkTests' tests "incus";

    in
    lxdTests // incusTests;

in
mkTests [
  ./tests/backup-and-prune
  ./tests/backup-and-prune-with-projects
  ./tests/backup-and-prune-with-remotes
  ./tests/dry-run
  ./tests/hooks
  ./tests/timeout
]
