main = MyMachine(main)
serverA = MyMachine(serverA)
serverB = MyMachine(serverB)
serverC = MyMachine(serverC)

# Setup serverA
serverA.succeed("lxc config set core.https_address 0.0.0.0")
serverA.succeed("lxc config set core.trust_password test")
serverA.succeed("date -s '2018-01-01 13:00:00'")
main.succeed("lxc remote add serverA 192.168.1.2 --accept-certificate --password test")

# Setup serverB
serverB.succeed("lxc config set core.https_address 0.0.0.0")
serverB.succeed("lxc config set core.trust_password test")
serverB.succeed("date -s '2018-01-01 13:00:00'")
main.succeed("lxc remote add serverB 192.168.1.3 --accept-certificate --password test")

# Setup serverC
serverC.succeed("lxc config set core.https_address 0.0.0.0")
serverC.succeed("lxc config set core.trust_password test")
serverC.succeed("date -s '2018-01-01 13:00:00'")
main.succeed("lxc remote add serverC 192.168.1.4 --accept-certificate --password test")

# Create containers
main.succeed("lxc launch image container")
serverA.succeed("lxc launch image container")
serverB.succeed("lxc launch image container")
serverC.succeed("lxc launch image container")

# Backup containers
main.lxd_snapper("backup", "expected.out.1.txt")
main.assert_snapshot_count("default", "container", ".*", 0)
serverA.assert_snapshot_count("default", "container", ".*", 1)
serverB.assert_snapshot_count("default", "container", ".*", 0)
serverC.assert_snapshot_count("default", "container", ".*", 1)

# Prune containers
main.lxd_snapper("prune", "expected.out.2.txt")
main.assert_snapshot_count("default", "container", ".*", 0)
serverA.assert_snapshot_count("default", "container", ".*", 0)
serverB.assert_snapshot_count("default", "container", ".*", 0)
serverC.assert_snapshot_count("default", "container", ".*", 0)
