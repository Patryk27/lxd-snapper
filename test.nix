import <nixpkgs/nixos/tests/make-test-python.nix> ({ pkgs, ... }:
  let
    alpine-meta = ./nix/lxd/alpine-meta.tar.xz;
    alpine-rootfs = ./nix/lxd/alpine-rootfs.tar.xz;
    lxd-config = ./nix/lxd/config.yaml;

    lxd-snapper = (import ./default.nix) + "/bin/lxd-snapper";
    lxd-snapper-config = ./nix/lxd-snapper/config.yaml;

  in
  {
    machine = { lib, ... }: {
      boot = {
        supportedFilesystems = [ "zfs" ];
      };

      networking = {
        hostId = "01234567";
      };

      virtualisation = {
        cores = 4;
        memorySize = 2048;

        lxd = {
          enable = true;
        };
      };
    };

    testScript = ''
      machine.wait_for_unit("multi-user.target")

      machine.succeed("truncate /var/tank -s 128MB")
      machine.succeed("zpool create tank /var/tank")

      machine.succeed(
          "cat ${lxd-config} | lxd init --preseed"
      )

      machine.succeed(
          "lxc image import ${alpine-meta} ${alpine-rootfs} --alias alpine"
      )

      with subtest("Create containers"):
          machine.succeed("lxc launch alpine mysql")
          machine.succeed("lxc snapshot mysql")

          machine.succeed("lxc launch alpine nginx")
          machine.succeed("lxc snapshot nginx")

          machine.succeed("lxc launch alpine php")
          machine.succeed("lxc snapshot php")

      with subtest("Smoke-test: validate"):
          assert "Everything seems to be fine" in machine.succeed(
              "${lxd-snapper} -c ${lxd-snapper-config} validate"
          )

      with subtest("Smoke-test: backup"):
          for (date, snapshot) in [
              ("2012-07-30 12:00:00", "auto-20120730-1200"),
              ("2012-07-31 12:00:00", "auto-20120731-1200"),
              ("2012-08-01 12:00:00", "auto-20120801-1200"),
              ("2012-08-02 12:00:00", "auto-20120802-1200"),
              ("2012-08-03 12:00:00", "auto-20120803-1200"),
              ("2012-08-04 12:00:00", "auto-20120804-1200"),
              ("2012-08-05 12:00:00", "auto-20120805-1200"),
              ("2012-08-06 12:00:00", "auto-20120806-1200"),
          ]:
              machine.succeed(f"date -s '{date}'")

              assert "created snapshots: 2" in machine.succeed(
                  "${lxd-snapper} -c ${lxd-snapper-config} backup"
              )

              for container in ["php", "mysql"]:
                  assert snapshot in machine.succeed(f"lxc info {container}")

              for container in ["nginx"]:
                  assert snapshot not in machine.succeed(f"lxc info {container}")

      with subtest("Smoke-test: prune"):
          machine.succeed(
              "${lxd-snapper} -c ${lxd-snapper-config} prune"
          )

          # TODO
    '';
  })
