# ====
#
# This script is part of lxd-snapper's integration tests - running it standalone
# will not work; please consult `../../README.md` for details.
#
# =====

machine.wait_for_unit("multi-user.target")
machine.wait_for_file("/var/lib/lxd/unix.socket")

machine.succeed("truncate /dev/shm/tank -s 128MB")
machine.succeed("zpool create tank /dev/shm/tank")

machine.succeed(
    "cat @lxd-config@ | lxd init --preseed"
)

machine.succeed(
    "lxc image import @lxd-alpine-meta@ @lxd-alpine-rootfs@ --alias alpine"
)


# Launches `lxd-snapper` with specified command and returns its output.
#
# ```python
# run_lxd_snapper("backup")
# ```
def run_lxd_snapper(cmd):
    return machine.succeed(
        f"@lxd-snapper@ -c @lxd-snapper-config@ {cmd}"
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
