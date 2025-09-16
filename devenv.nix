{ pkgs, lib, config, inputs, ... }:

{
  # https://devenv.sh/packages/
  packages = with pkgs; [ git libyaml openssl zlib.static glibc glibc.static cargo-zigbuild cmake libgcc
    libclang
    pkg-config
  ];

  languages.rust = {
    enable = true;
    channel = "nightly";
    targets = [
      "x86_64-unknown-linux-gnu"
      "x86_64-unknown-linux-musl"
      "aarch64-unknown-linux-gnu"
      "aarch64-unknown-linux-musl"
      "x86_64-pc-windows-gnu"
      "aarch64-pc-windows-msvc"
      "x86_64-apple-darwin"
      "aarch64-apple-darwin"
    ];
  };
  languages.zig.enable = true;


  enterShell = ''

  '';

}
