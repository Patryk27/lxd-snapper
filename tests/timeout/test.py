machine = MyMachine(machine)
machine.succeed("lxc-or-incus launch image test")
machine.lxd_snapper_err("backup", "expected.out.txt")
