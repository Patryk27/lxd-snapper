{ nixpkgs, lxd-snapper }:

let
  pkgs = import nixpkgs {
    # Even though we do support i686, lxd depends on criu which doesn't work
    # there, so x86_64 it is
    system = "x86_64-linux";
  };

  lxd-image = import "${nixpkgs}/nixos/release.nix" {
    configuration = {
      documentation.enable = pkgs.lib.mkForce false;
    };
  };

  mkTest = name:
    let
      testScriptCommon = import ./cases/common.nix {
        lxd-config = ./fixtures/lxd-config.yaml;
        lxd-image-metadata = lxd-image.lxdMeta.${pkgs.system};
        lxd-image-rootfs = lxd-image.lxdImage.${pkgs.system};
        test = "${./cases}/${name}";
      };

      testScript =
        testScriptCommon
        + "\n\n"
        + (builtins.readFile "${./cases}/${name}/test.py");

    in
    pkgs.nixosTest {
      inherit name testScript;

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
              package = pkgs.lxd;
            };
          };
        };
      };
    };

in
{
  backup-and-prune = mkTest "backup-and-prune";
  backup-and-prune-with-projects = mkTest "backup-and-prune-with-projects";
  dry-run = mkTest "dry-run";
  hooks = mkTest "hooks";
}
