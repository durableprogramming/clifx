{ pkgs, lib, config, inputs, ... }:

{
  # https://devenv.sh/packages/
  packages = with pkgs; [ git libyaml openssl glibc.static zlib.static glibc];

  languages.rust.enable = true;


  enterShell = ''

    export RUSTFLAGS="-C target-feature=-crt-static"
  '';

}
