import <nixpkgs/nixos/tests/make-test-python.nix> ({ pkgs, ... }:
  let
    alpine-meta = ./nix/lxd/alpine-meta.tar.xz;
    alpine-rootfs = ./nix/lxd/alpine-rootfs.tar.xz;
    lxd-config = ./nix/lxd/config.yaml;

    lxd-snapper = (import ./default.nix) + "/bin/lxd-snapper";
    lxd-snapper-config = ./nix/lxd-snapper/config.yaml;

  in
  {
    machine = { lib, pkgs, ... }: {
      boot = {
        supportedFilesystems = [ "zfs" ];
      };

      environment = {
        systemPackages = with pkgs; [
          jq
        ];
      };

      networking = {
        # Required for ZFS; value doesn't matter.
        hostId = "01234567";
      };

      virtualisation = {
        # We're using extra cores and memory for the tests to be faster; in a
        # more constrained environment, 1 core and 512 MB of RAM should be
        # enough though
        cores = 4;
        memorySize = 2048;

        lxd = {
          enable = true;
        };
      };
    };

    testScript = ''
      machine.wait_for_unit("multi-user.target")
      machine.wait_for_file("/var/lib/lxd/unix.socket")

      machine.succeed("truncate /dev/shm/tank -s 128MB")
      machine.succeed("zpool create tank /dev/shm/tank")

      machine.succeed(
          "cat ${lxd-config} | lxd init --preseed"
      )

      machine.succeed(
          "lxc image import ${alpine-meta} ${alpine-rootfs} --alias alpine"
      )


      # Starts `lxd-snapper` with the default configuration and specified
      # command, and returns its output.
      #
      # ```python
      # run_lxd_snapper("backup")
      # ```
      def run_lxd_snapper(cmd):
          return machine.succeed(
              f"${lxd-snapper} -c ${lxd-snapper-config} {cmd}"
          )


      # Asserts that given container contains exactly `count` snapshots matching
      # given regex.
      #
      # ```python
      # assert_snapshot_count("test", "snap\d", 1)
      # ```
      def assert_snapshot_count(container, snapshot_regex, snapshot_count):
          snapshot_regex = snapshot_regex.replace("\\", "\\\\")

          machine.succeed(
              f"lxc query /1.0/instances/{container}/snapshots"
              + f" | jq -e '[ .[] | select(test(\"{snapshot_regex}\")) ] | length == {snapshot_count}'"
          )


      # Asserts that given container contains exactly one snapshot matching
      # given regex.
      #
      # ```python
      # assert_snapshot_exists("test", "snap\d")
      # ```
      def assert_snapshot_exists(container, snapshot_regex):
          assert_snapshot_count(container, snapshot_regex, 1)


      # Asserts that given container contains exactly zero snapshots matching
      # given regex.
      #
      # ```python
      # assert_snapshot_does_not_exist("test", "snap\d")
      # ```
      def assert_snapshot_does_not_exist(container, snapshot_regex):
          assert_snapshot_count(container, snapshot_regex, 0)


      with subtest("Create some containers and manual snapshots"):
          machine.succeed("lxc launch alpine mysql")
          machine.succeed("lxc snapshot mysql")
          assert_snapshot_exists("mysql", "snap0")

          machine.succeed("lxc launch alpine nginx")
          machine.succeed("lxc snapshot nginx")
          assert_snapshot_exists("nginx", "snap0")

          machine.succeed("lxc launch alpine php")
          machine.succeed("lxc snapshot php")
          assert_snapshot_exists("php", "snap0")

      with subtest("Smoke-test: validate"):
          assert "Everything seems to be fine" in run_lxd_snapper("validate")

      with subtest("Smoke-test: backup"):
          for (date, snapshot_regex) in [
              ("2012-07-30 12:00:00", "auto\-20120730\-1200\d{2}"),
              ("2012-07-31 12:00:00", "auto\-20120731\-1200\d{2}"),
              ("2012-08-01 12:00:00", "auto\-20120801\-1200\d{2}"),
              ("2012-08-02 12:00:00", "auto\-20120802\-1200\d{2}"),
              ("2012-08-03 12:00:00", "auto\-20120803\-1200\d{2}"),
              ("2012-08-04 12:00:00", "auto\-20120804\-1200\d{2}"),
              ("2012-08-05 12:00:00", "auto\-20120805\-1200\d{2}"),
              ("2012-08-06 12:00:00", "auto\-20120806\-1200\d{2}"),
          ]:
              machine.succeed(f"date -s '{date}'")

              assert "created snapshots: 2" in run_lxd_snapper("backup")

              for container in ["php", "mysql"]:
                  assert_snapshot_exists(container, snapshot_regex)

              for container in ["nginx"]:
                  assert_snapshot_does_not_exist(container, snapshot_regex)

      with subtest("Smoke-test: prune"):
          out = run_lxd_snapper("prune")

          assert "processed containers: 3" in out
          assert "deleted snapshots: 8" in out
          assert "kept snapshots: 8" in out

          # Perform assertions for the `mysql` container
          assert_snapshot_exists("mysql", "snap0")
          assert_snapshot_does_not_exist("mysql", "auto\-20120730\-1200\d{2}")
          assert_snapshot_exists("mysql", "auto\-20120731\-1200\d{2}")
          assert_snapshot_does_not_exist("mysql", "auto\-20120801\-1200\d{2}")
          assert_snapshot_exists("mysql", "auto\-20120802\-1200\d{2}")
          assert_snapshot_exists("mysql", "auto\-20120803\-1200\d{2}")
          assert_snapshot_exists("mysql", "auto\-20120804\-1200\d{2}")
          assert_snapshot_exists("mysql", "auto\-20120805\-1200\d{2}")
          assert_snapshot_exists("mysql", "auto\-20120806\-1200\d{2}")
          assert_snapshot_count("mysql", ".*", 7)  # 1 manual + 6 automatic

          # Perform assertions for the `nginx` container
          assert_snapshot_count("nginx", ".*", 1)  # 1 manual

          # Perform assertions for the `php` container
          assert_snapshot_exists("php", "snap0")
          assert_snapshot_does_not_exist("php", "auto\-20120730\-1200\d{2}")
          assert_snapshot_does_not_exist("php", "auto\-20120731\-1200\d{2}")
          assert_snapshot_does_not_exist("php", "auto\-20120801\-1200\d{2}")
          assert_snapshot_does_not_exist("php", "auto\-20120802\-1200\d{2}")
          assert_snapshot_does_not_exist("php", "auto\-20120803\-1200\d{2}")
          assert_snapshot_does_not_exist("php", "auto\-20120804\-1200\d{2}")
          assert_snapshot_exists("php", "auto\-20120806\-1200\d{2}")
          assert_snapshot_exists("php", "auto\-20120805\-1200\d{2}")
          assert_snapshot_count("php", ".*", 3)  # 1 manual + 2 automatic
    '';
  })
