machine = MyMachine(machine)
machine.succeed("lxc launch image test")

machine.lxd_snapper("--dry-run backup", "expected.out.1.txt")
machine.assert_snapshot_does_not_exist("default", "test", "auto\-.*")

machine.lxd_snapper("backup", "expected.out.2.txt")
machine.assert_snapshot_exists("default", "test", "auto\-.*")

machine.lxd_snapper("--dry-run prune", "expected.out.3.txt")
machine.assert_snapshot_exists("default", "test", "auto\-.*")

machine.lxd_snapper("prune", "expected.out.4.txt")
machine.assert_snapshot_does_not_exist("default", "test", "auto\-.*")
