machine = MyMachine(machine)
machine.succeed("lxc-or-incus launch image test")

machine.lxd_snapper("backup", "expected.out.1.txt")
machine.assert_snapshot_exists("default", "test", "auto\-.*")

machine.lxd_snapper("prune", "expected.out.2.txt")
machine.assert_snapshot_does_not_exist("default", "test", "auto\-.*")
