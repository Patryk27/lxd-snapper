# ====
#
# This script is part of lxd-snapper's integration tests - running it standalone
# will not work; please consult `../../../README.md` for details.
#
# =====

machine.succeed("lxc project switch default")
machine.succeed("lxc launch alpine test")

with subtest("Test: Simulated backup"):
    out = run_lxd_snapper("--dry-run backup")

    assert (
        "--dry-run is active" in out
    ), f"missing --dry-run hint; actual output: {out}"

    assert (
        "created snapshots: 1" in out
    ), f"created snapshots != 1; actual output: {out}"

    assert_snapshot_does_not_exist("default", "test", "auto\-.*")

with subtest("Test: Real backup"):
    out = run_lxd_snapper("backup")

    assert (
        "created snapshots: 1" in out
    ), f"created snapshots != 1; actual output: {out}"

    assert_snapshot_exists("default", "test", "auto\-.*")

with subtest("Test: Simulated prune"):
    out = run_lxd_snapper("--dry-run prune")

    assert (
        "--dry-run is active" in out
    ), f"missing --dry-run hint; actual output: {out}"

    assert (
        "deleted snapshots: 1" in out
    ), f"deleted snapshots != 1; actual output: {out}"

    assert_snapshot_exists("default", "test", "auto\-.*")

with subtest("Test: Real prune"):
    out = run_lxd_snapper("prune")

    assert (
        "deleted snapshots: 1" in out
    ), f"deleted snapshots != 1; actual output: {out}"

    assert_snapshot_does_not_exist("default", "test", "auto\-.*")
