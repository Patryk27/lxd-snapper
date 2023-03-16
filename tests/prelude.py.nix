{ lxdConfig, lxdImageMetadata, lxdImageRootfs, testPath }: ''
  class MyMachine(Machine):
      def __init__(self, base):
          base.succeed("date -s '2018-01-01 12:00:00'")
          base.wait_for_unit("multi-user.target")
          base.wait_for_file("/var/lib/lxd/unix.socket")

          base.succeed("mkdir /test")
          base.succeed("mount --bind ${testPath} /test")
          base.succeed("truncate /dev/shm/tank -s 1024MB")
          base.succeed("zpool create tank /dev/shm/tank")

          base.succeed(
              "cat ${lxdConfig} | lxd init --preseed"
          )

          base.succeed(
              "lxc image import ${lxdImageMetadata}/*/*.tar.xz ${lxdImageRootfs}/*/*.tar.xz --alias image"
          )

          self.base = base


      def succeed(self, command):
          return self.base.succeed(command)


      def fail(self, command):
          return self.base.fail(command)


      # Launches `lxd-snapper` with specified command, asserts that it succeeded
      # and returns its output.
      #
      # ```python
      # machine.lxd_snapper("backup")
      # ```
      def lxd_snapper(self, cmd, expected_out_file = None):
          actual_out = self.succeed(
              f"lxd-snapper -c /test/config.yaml {cmd}"
          )

          if expected_out_file:
              expected_out = self.succeed(f"cat /test/{expected_out_file}")
              assert expected_out == actual_out, f"outputs don't match; actual output:\n{actual_out}"

          return actual_out


      # Launches `lxd-snapper` with specified command, asserts that it failed
      # and returns its output.
      #
      # ```python
      # machine.lxd_snapper_err("backup")
      # ```
      def lxd_snapper_err(self, cmd, expected_out_file = None):
          actual_out = self.fail(
              f"lxd-snapper -c /test/config.yaml {cmd}"
          )

          if expected_out_file:
              expected_out = self.succeed(f"cat /test/{expected_out_file}")
              assert expected_out == actual_out, f"outputs don't match; actual output:\n{actual_out}"

          return actual_out


      # Asserts that given instance contains exactly `count` snapshots matching
      # given regex.
      #
      # ```python
      # machine.assert_snapshot_count("default", "test", "snap\d", 1)
      # ```
      def assert_snapshot_count(self, project, instance, snapshot_regex, snapshot_count):
          snapshot_regex = snapshot_regex.replace("\\", "\\\\")

          self.succeed(
              f"lxc query /1.0/instances/{instance}/snapshots?project={project}"
              + f" | jq -e '[ .[] | select(test(\"{snapshot_regex}\")) ] | length == {snapshot_count}'"
          )


      # Asserts that given instance contains exactly one snapshot matching given
      # regex.
      #
      # ```python
      # machine.assert_snapshot_exists("default", "test", "snap\d")
      # ```
      def assert_snapshot_exists(self, project, instance, snapshot_regex):
          self.assert_snapshot_count(project, instance, snapshot_regex, 1)


      # Asserts that given instance contains exactly zero snapshots matching given
      # regex.
      #
      # ```python
      # machine.assert_snapshot_does_not_exist("default", "test", "snap\d")
      # ```
      def assert_snapshot_does_not_exist(self, project, instance, snapshot_regex):
          self.assert_snapshot_count(project, instance, snapshot_regex, 0)
''
