{ nixpkgs, lxd-snapper }:
let
  pkgs = import nixpkgs {
    # Even though we do support i686, lxd depends on criu which doesn't work
    # there, so x86_64 it is
    system = "x86_64-linux";
  };

  # During the tests we don't have access to the internet, so we have to use
  # pre-downloaded container images
  lxd-alpine-meta = "${./fixtures/lxd-alpine-meta.tar.xz}";
  lxd-alpine-rootfs = "${./fixtures/lxd-alpine-rootfs.tar.xz}";
  lxd-config = "${./fixtures/lxd-config.yaml}";

  mkTest = name:
    let
      test = ./cases + "/${name}/test.py";

      testScript' =
        (builtins.readFile ./cases/common.py) + "\n\n" + (builtins.readFile test);

      testScript =
        builtins.replaceStrings
          [
            "@lxd-alpine-meta@"
            "@lxd-alpine-rootfs@"
            "@lxd-config@"
            "@lxd-snapper@"
            "@dir@"
          ]
          [
            lxd-alpine-meta
            lxd-alpine-rootfs
            lxd-config
            "${lxd-snapper}/bin/lxd-snapper"
            "${./cases}/${name}"
          ]
          testScript';

    in
    pkgs.nixosTest ({
      inherit name testScript;

      machine = { ... }: {
        boot = {
          supportedFilesystems = [ "zfs" ];
        };

        environment = {
          systemPackages = with pkgs; [
            # Required for our testScript
            jq
          ];
        };

        networking = {
          # Required for ZFS; value doesn't matter
          hostId = "01234567";
        };

        virtualisation = {
          # Neither lxd-snapper nor LXD require so much resources, but using a
          # little bit more CPU and RAM makes the tests go faster
          cores = 2;
          memorySize = 512;

          lxd = {
            enable = true;
            package = pkgs.lxd;
          };
        };
      };
    });
in
{
  backup-and-prune = mkTest "backup-and-prune";
  backup-and-prune-with-projects = mkTest "backup-and-prune-with-projects";
  dry-run = mkTest "dry-run";
  hooks = mkTest "hooks";
}
