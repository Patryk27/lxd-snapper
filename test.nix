# This file contains an integration test for lxd-snapper that spawns a virtual machine with LXD in
# scope and performs a couple of smoke tests.
#
# Since we piggy-back on the `nixos/tests` facilities, this test doesn't have any implicit
# dependencies, nor does it alter any machine's state in any way.
#
# To launch tests, please run:
# $ nix-build ./test.nix
#
# On a more or less recent computer, all tests should complete in around 3 minutes.
let
  host-pkgs = import <nixpkgs> { };

  launch-test = { pkgs }: import <nixpkgs/nixos/tests/make-test-python.nix>
    ({ ... }:
      let
        # Since our virtual machine doesn't have access to the internet, we have to use
        # pre-downloaded container images.
        #
        # I've settled on Alpine Linux only because its images are tiny - when push comes to shove,
        # the concrete distribution doesn't matter.
        lxd-alpine-meta = ./nix/lxd/alpine-meta.tar.xz;
        lxd-alpine-rootfs = ./nix/lxd/alpine-rootfs.tar.xz;

        lxd-config = ./nix/lxd/config.yaml;

        lxd-snapper = (import ./default.nix) + "/bin/lxd-snapper";
        lxd-snapper-config = ./nix/lxd-snapper/config.yaml;

      in
      {
        machine = { ... }: {
          boot = {
            supportedFilesystems = [ "zfs" ];
          };

          environment = {
            systemPackages = with host-pkgs; [
              jq
            ];
          };

          networking = {
            # Required for ZFS; value doesn't matter
            hostId = "01234567";
          };

          virtualisation = {
            # Neither lxd-snapper nor LXD require so much resources, but using a little bit more
            # CPU & RAM makes the tests go faster
            cores = 2;
            memorySize = 512;

            lxd = {
              enable = true;
              package = pkgs.lxd;
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
              "lxc image import ${lxd-alpine-meta} ${lxd-alpine-rootfs} --alias alpine"
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


          # Asserts that given instance contains exactly `count` snapshots matching
          # given regex.
          #
          # ```python
          # assert_snapshot_count("default", "test", "snap\d", 1)
          # ```
          def assert_snapshot_count(project, instance, snapshot_regex, snapshot_count):
              snapshot_regex = snapshot_regex.replace("\\", "\\\\")

              machine.succeed(
                  f"lxc query /1.0/instances/{instance}/snapshots?project={project}"
                  + f" | jq -e '[ .[] | select(test(\"{snapshot_regex}\")) ] | length == {snapshot_count}'"
              )


          # Asserts that given instance contains exactly one snapshot matching
          # given regex.
          #
          # ```python
          # assert_snapshot_exists("default", "test", "snap\d")
          # ```
          def assert_snapshot_exists(project, instance, snapshot_regex):
              assert_snapshot_count(project, instance, snapshot_regex, 1)


          # Asserts that given instance contains exactly zero snapshots matching
          # given regex.
          #
          # ```python
          # assert_snapshot_does_not_exist("default", "test", "snap\d")
          # ```
          def assert_snapshot_does_not_exist(project, instance, snapshot_regex):
              assert_snapshot_count(project, instance, snapshot_regex, 0)


          with subtest("Create some instances and snapshots for client-a"):
              machine.succeed(
                  "lxc project create client-a -c features.images=false -c features.profiles=false"
              )

              machine.succeed("lxc project switch client-a")

              machine.succeed("lxc launch alpine apache")
              machine.succeed("lxc snapshot apache")
              assert_snapshot_exists("client-a", "apache", "snap0")
              assert_snapshot_count("client-a", "apache", ".*", 1)

              machine.succeed("lxc launch alpine mysql")
              machine.succeed("lxc snapshot mysql")
              assert_snapshot_exists("client-a", "mysql", "snap0")
              assert_snapshot_count("client-a", "mysql", ".*", 1)

              machine.succeed("lxc launch alpine php")
              machine.succeed("lxc snapshot php")
              assert_snapshot_exists("client-a", "php", "snap0")
              assert_snapshot_count("client-a", "php", ".*", 1)

          with subtest("Create some instances for client-b"):
              machine.succeed(
                  "lxc project create client-b -c features.images=false -c features.profiles=false"
              )

              machine.succeed("lxc project switch client-b")

              machine.succeed("lxc launch alpine apache")
              assert_snapshot_count("client-b", "apache", ".*", 0)

              machine.succeed("lxc launch alpine mysql")
              assert_snapshot_count("client-b", "mysql", ".*", 0)

              machine.succeed("lxc launch alpine php")
              assert_snapshot_count("client-b", "php", ".*", 0)

          with subtest("Create some instances for client-c"):
              machine.succeed(
                  "lxc project create client-c -c features.images=false -c features.profiles=false"
              )

              machine.succeed("lxc project switch client-c")

              machine.succeed("lxc launch alpine apache")
              assert_snapshot_count("client-c", "apache", ".*", 0)

              machine.succeed("lxc launch alpine mysql")
              assert_snapshot_count("client-c", "mysql", ".*", 0)

              machine.succeed("lxc launch alpine php")
              assert_snapshot_count("client-c", "php", ".*", 0)

          machine.succeed("lxc project switch default")

          with subtest("Smoke-test: validate"):
              out = run_lxd_snapper("validate")

              assert "Everything seems to be fine" in out, f"actual output: {out}"

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

                  out = run_lxd_snapper("backup")

                  assert (
                      "created snapshots: 4" in out
                  ), f"created snapshots != 4; actual output: {out}`"

                  for project in ["client-a", "client-b"]:
                      assert_snapshot_does_not_exist(project, "apache", snapshot_regex)
                      assert_snapshot_exists(project, "mysql", snapshot_regex)
                      assert_snapshot_exists(project, "php", snapshot_regex)

                  for project in ["client-c"]:
                      assert_snapshot_does_not_exist(project, "apache", snapshot_regex)
                      assert_snapshot_does_not_exist(project, "mysql", snapshot_regex)
                      assert_snapshot_does_not_exist(project, "php", snapshot_regex)

          with subtest("Smoke-test: prune"):
              out = run_lxd_snapper("prune")

              assert (
                  "processed instances: 9" in out
              ), f"processed instances != 9; actual output: {out}"

              assert (
                  "deleted snapshots: 16" in out
              ), f"deleted snapshots != 16; actual output: {out}"

              assert "kept snapshots: 16" in out, f"kept snapshots != 16; actual output: {out}"

              for (project, manual_snapshot_count) in [("client-a", 1), ("client-b", 0)]:
                  # While starting the test, we've created a manual snapshot (via `lxc snapshot`) for
                  # each container inside the `client-a` project. Since those snapshots are - like I
                  # said - manual, they shouldn't be touched by the `prune` command
                  if manual_snapshot_count > 0:
                      assert_snapshot_exists(project, "mysql", "snap0")
                      assert_snapshot_exists(project, "php", "snap0")

                  assert_snapshot_does_not_exist(project, "mysql", "auto\-20120730\-1200\d{2}")
                  assert_snapshot_exists(project, "mysql", "auto\-20120731\-1200\d{2}")
                  assert_snapshot_does_not_exist(project, "mysql", "auto\-20120801\-1200\d{2}")
                  assert_snapshot_exists(project, "mysql", "auto\-20120802\-1200\d{2}")
                  assert_snapshot_exists(project, "mysql", "auto\-20120803\-1200\d{2}")
                  assert_snapshot_exists(project, "mysql", "auto\-20120804\-1200\d{2}")
                  assert_snapshot_exists(project, "mysql", "auto\-20120805\-1200\d{2}")
                  assert_snapshot_exists(project, "mysql", "auto\-20120806\-1200\d{2}")

                  assert_snapshot_does_not_exist(project, "php", "auto\-20120730\-1200\d{2}")
                  assert_snapshot_does_not_exist(project, "php", "auto\-20120731\-1200\d{2}")
                  assert_snapshot_does_not_exist(project, "php", "auto\-20120801\-1200\d{2}")
                  assert_snapshot_does_not_exist(project, "php", "auto\-20120802\-1200\d{2}")
                  assert_snapshot_does_not_exist(project, "php", "auto\-20120803\-1200\d{2}")
                  assert_snapshot_does_not_exist(project, "php", "auto\-20120804\-1200\d{2}")
                  assert_snapshot_exists(project, "php", "auto\-20120806\-1200\d{2}")
                  assert_snapshot_exists(project, "php", "auto\-20120805\-1200\d{2}")

                  assert_snapshot_count(project, "apache", ".*", manual_snapshot_count)
                  assert_snapshot_count(project, "mysql", ".*", manual_snapshot_count + 6)
                  assert_snapshot_count(project, "php", ".*", manual_snapshot_count + 2)

              for project in ["client-c"]:
                  for container in ["apache", "mysql", "php"]:
                      assert_snapshot_count(project, container, ".*", 0)
        '';
      }) { };

  launch-test-for-older-nixpkgs = { rev, sha256 }: launch-test {
    pkgs = import
      (host-pkgs.fetchFromGitHub {
        owner = "nixos";
        repo = "nixpkgs";
        inherit rev;
        inherit sha256;
      }) { };
  };

in
{
  lxd-4-0 = launch-test-for-older-nixpkgs {
    rev = "79374b98700366f0aa39241c05a840c4c221c1c0";
    sha256 = "0dnpc889ccy7hcwkpks29m1jjwhklxrfzdqdpb1a693q8wn54c5j";
  };

  lxd-4-1 = launch-test-for-older-nixpkgs {
    rev = "528d35bec0cb976a06cc0e8487c6e5136400b16b";
    sha256 = "0j8mnzq2c0rrd1s8whs8vc5aj3v8kn85ifzkn1z77ihvxbj5cn7y";
  };

  lxd-4-2 = launch-test-for-older-nixpkgs {
    rev = "fa54dd346fe5e73d877f2068addf6372608c820b";
    sha256 = "1428qilgk9h9w0lka0xmjnrkllyz16kny1afz3asr0qnr63wyzdk";
  };

  lxd = launch-test {
    pkgs = host-pkgs;
  };
}
