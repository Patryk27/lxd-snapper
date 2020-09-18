(
  import
    (
      fetchTarball {
        url = "https://github.com/edolstra/flake-compat/archive/e363cffac26b0009972e0e20c1bbb0953473bd86.tar.gz";
        sha256 = "0h0iw41nbrarz1n39f0f94xkg4gjvl2vlhlqkivmbwrib5jwspnj";
      }
    ) {
    src = ./.;
  }
).defaultNix
