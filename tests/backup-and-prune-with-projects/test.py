machine = MyMachine(machine)

with subtest("Create some instances and snapshots for client-a"):
    machine.succeed(
        "lxc project create client-a -c features.images=false -c features.profiles=false"
    )

    machine.succeed("lxc project switch client-a")

    machine.succeed("lxc launch image apache")
    machine.succeed("lxc snapshot apache")
    machine.assert_snapshot_exists("client-a", "apache", "snap0")
    machine.assert_snapshot_count("client-a", "apache", ".*", 1)

    machine.succeed("lxc launch image mysql")
    machine.succeed("lxc snapshot mysql")
    machine.assert_snapshot_exists("client-a", "mysql", "snap0")
    machine.assert_snapshot_count("client-a", "mysql", ".*", 1)

    machine.succeed("lxc launch image php")
    machine.succeed("lxc snapshot php")
    machine.assert_snapshot_exists("client-a", "php", "snap0")
    machine.assert_snapshot_count("client-a", "php", ".*", 1)


with subtest("Create some instances for client-b"):
    machine.succeed(
        "lxc project create client-b -c features.images=false -c features.profiles=false"
    )

    machine.succeed("lxc project switch client-b")

    machine.succeed("lxc launch image apache")
    machine.assert_snapshot_count("client-b", "apache", ".*", 0)

    machine.succeed("lxc launch image mysql")
    machine.assert_snapshot_count("client-b", "mysql", ".*", 0)

    machine.succeed("lxc launch image php")
    machine.assert_snapshot_count("client-b", "php", ".*", 0)


with subtest("Create some instances for client-c"):
    machine.succeed(
        "lxc project create client-c -c features.images=false -c features.profiles=false"
    )

    machine.succeed("lxc project switch client-c")

    machine.succeed("lxc launch image apache")
    machine.assert_snapshot_count("client-c", "apache", ".*", 0)

    machine.succeed("lxc launch image mysql")
    machine.assert_snapshot_count("client-c", "mysql", ".*", 0)

    machine.succeed("lxc launch image php")
    machine.assert_snapshot_count("client-c", "php", ".*", 0)


machine.succeed("lxc project switch default")


with subtest("Backup"):
    for (date, snapshot_regex) in [
        ("2012-07-30 12:00:00", "auto\-20120730"),
        ("2012-07-31 12:00:00", "auto\-20120731"),
        ("2012-08-01 12:00:00", "auto\-20120801"),
        ("2012-08-02 12:00:00", "auto\-20120802"),
        ("2012-08-03 12:00:00", "auto\-20120803"),
        ("2012-08-04 12:00:00", "auto\-20120804"),
        ("2012-08-05 12:00:00", "auto\-20120805"),
        ("2012-08-06 12:00:00", "auto\-20120806"),
    ]:
        machine.succeed(f"date -s '{date}'")

        out = machine.lxd_snapper("backup")

        assert (
            "created snapshots: 4" in out
        ), f"created snapshots != 4; actual output: {out}"

        for project in ["client-a", "client-b"]:
            machine.assert_snapshot_does_not_exist(project, "apache", snapshot_regex)
            machine.assert_snapshot_exists(project, "mysql", snapshot_regex)
            machine.assert_snapshot_exists(project, "php", snapshot_regex)

        for project in ["client-c"]:
            machine.assert_snapshot_does_not_exist(project, "apache", snapshot_regex)
            machine.assert_snapshot_does_not_exist(project, "mysql", snapshot_regex)
            machine.assert_snapshot_does_not_exist(project, "php", snapshot_regex)


with subtest("Prune"):
    out = machine.lxd_snapper("prune")

    assert (
        "processed instances: 4" in out
    ), f"processed instances != 4; actual output: {out}"

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
            machine.assert_snapshot_exists(project, "mysql", "snap0")
            machine.assert_snapshot_exists(project, "php", "snap0")

        machine.assert_snapshot_does_not_exist(project, "mysql", "auto\-20120730")
        machine.assert_snapshot_exists(project, "mysql", "auto\-20120731")
        machine.assert_snapshot_does_not_exist(project, "mysql", "auto\-20120801")
        machine.assert_snapshot_exists(project, "mysql", "auto\-20120802")
        machine.assert_snapshot_exists(project, "mysql", "auto\-20120803")
        machine.assert_snapshot_exists(project, "mysql", "auto\-20120804")
        machine.assert_snapshot_exists(project, "mysql", "auto\-20120805")
        machine.assert_snapshot_exists(project, "mysql", "auto\-20120806")

        machine.assert_snapshot_does_not_exist(project, "php", "auto\-20120730")
        machine.assert_snapshot_does_not_exist(project, "php", "auto\-20120731")
        machine.assert_snapshot_does_not_exist(project, "php", "auto\-20120801")
        machine.assert_snapshot_does_not_exist(project, "php", "auto\-20120802")
        machine.assert_snapshot_does_not_exist(project, "php", "auto\-20120803")
        machine.assert_snapshot_does_not_exist(project, "php", "auto\-20120804")
        machine.assert_snapshot_exists(project, "php", "auto\-20120806")
        machine.assert_snapshot_exists(project, "php", "auto\-20120805")

        machine.assert_snapshot_count(project, "apache", ".*", manual_snapshot_count)
        machine.assert_snapshot_count(project, "mysql", ".*", manual_snapshot_count + 6)
        machine.assert_snapshot_count(project, "php", ".*", manual_snapshot_count + 2)

    for project in ["client-c"]:
        for instance in ["apache", "mysql", "php"]:
            machine.assert_snapshot_count(project, instance, ".*", 0)
