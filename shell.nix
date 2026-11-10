let
  pkgs = import (fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/1c89083e451e891152365b799595cf7790a43325.tar.gz";
    sha256 = "0gnw7gkrh9axnknb67627mf9sbdd7sbgw3148g5lh1d8hiks4l33";
  }) { };
  ocamlPackages = pkgs.ocamlPackages;
  wasm = ocamlPackages.buildDunePackage {
    pname = "wasm";
    version = "3.0";
    src = pkgs.fetchFromGitHub {
      owner = "WebAssembly";
      repo = "spec";
      rev = "9d36019973201a19f9c9ebb0f10828b2fe2374aa";
      hash = "sha256-02gBWWf8usPamYwT4nYy+symbO2zoXyUxClchwaWhk8=";
    };
    sourceRoot = "source/interpreter";
    nativeBuildInputs = [ ocamlPackages.menhir ];
  };
in
pkgs.mkShell {
  PYTHONDONTWRITEBYTECODE = 1;
  packages = with ocamlPackages; [ ocaml dune_3 findlib cmdliner wasm pkgs.ocamlformat pkgs.python3 pkgs.librsvg pkgs.luajit pkgs.emscripten pkgs.binaryen ];
}
