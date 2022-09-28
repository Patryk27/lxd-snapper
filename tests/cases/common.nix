{ lxd-config, lxd-image-metadata, lxd-image-rootfs, test }: ''
  machine.wait_for_unit("multi-user.target")
  machine.wait_for_file("/var/lib/lxd/unix.socket")

  machine.succeed("mkdir /test")
  machine.succeed("mount --bind ${test} /test")

  machine.succeed("truncate /dev/shm/tank -s 4096MB")
  machine.succeed("zpool create tank /dev/shm/tank")

  machine.succeed(
      "cat ${lxd-config} | lxd init --preseed"
  )

  machine.succeed(
      "lxc image import ${lxd-image-metadata}/*/*.tar.xz ${lxd-image-rootfs}/*/*.tar.xz --alias image"
  )


  # Launches `lxd-snapper` with specified command, asserts it succeeded and
  # returns its output.
  #
  # ```python
  # run("backup")
  # ```
  def run(cmd):
      return machine.succeed(
          f"lxd-snapper -c /test/config.yaml {cmd}"
      )


  # Asserts that given instance contains exactly `count` snapshots matching given
  # regex.
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


  # Asserts that given instance contains exactly one snapshot matching given
  # regex.
  #
  # ```python
  # assert_snapshot_exists("default", "test", "snap\d")
  # ```
  def assert_snapshot_exists(project, instance, snapshot_regex):
      assert_snapshot_count(project, instance, snapshot_regex, 1)


  # Asserts that given instance contains exactly zero snapshots matching given
  # regex.
  #
  # ```python
  # assert_snapshot_does_not_exist("default", "test", "snap\d")
  # ```
  def assert_snapshot_does_not_exist(project, instance, snapshot_regex):
      assert_snapshot_count(project, instance, snapshot_regex, 0)
''
