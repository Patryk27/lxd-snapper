# ====
#
# This script is part of lxd-snapper's integration tests - running it standalone
# will not work; please consult `../../../README.md` for details.
#
# =====

with subtest("Fixture: Create some instances and snapshots for client-a"):
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

with subtest("Fixture: Create some instances for client-b"):
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

with subtest("Fixture: Create some instances for client-c"):
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

with subtest("Test: Backup"):
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
        ), f"created snapshots != 4; actual output: {out}"

        for project in ["client-a", "client-b"]:
            assert_snapshot_does_not_exist(project, "apache", snapshot_regex)
            assert_snapshot_exists(project, "mysql", snapshot_regex)
            assert_snapshot_exists(project, "php", snapshot_regex)

        for project in ["client-c"]:
            assert_snapshot_does_not_exist(project, "apache", snapshot_regex)
            assert_snapshot_does_not_exist(project, "mysql", snapshot_regex)
            assert_snapshot_does_not_exist(project, "php", snapshot_regex)

with subtest("Test: Prune"):
    out = run_lxd_snapper("prune")

    assert (
        "processed instances: 9" in out
    ), f"processed instances != 9; actual output: {out}"

    assert (
        "deleted snapshots: 16" in out
    ), f"deleted snapshots != 16; actual output: {out}"

    assert "kept snapshots: 16" in out, f"kept snapshots != 16; actual output: {out}"

    for (project, manual_snapshot_count) in [("client-a", 1), ("client-b", 0)]:
        # While starting the test, we've created a manual snapshot (via `lxc
        # snapshot`) for each instance inside the `client-a` project.
        #
        # Since those snapshots are manual, they shouldn't be touched by the
        # `prune` command.
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
        for instance in ["apache", "mysql", "php"]:
            assert_snapshot_count(project, instance, ".*", 0)
