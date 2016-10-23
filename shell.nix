{ pkgs ? import <nixpkgs> {} }:

with pkgs;

let

  stdenv = pkgs.stdenv;
  deps = [ openssl ];
  f = { rustc, cargo, openssl}:
  stdenv.mkDerivation rec {
    name = "wop";
    version = "0.1.0";
    src = ./.;
    buildInputs = [ rustc cargo openssl ];
    LIBRARY_PATH = stdenv.lib.makeLibraryPath ( deps );
    LDFLAGS = "-L${openssl}/lib";
    CFLAGS = "-I${openssl}/include";
  };
  drv = callPackage f {};

in
  drv

