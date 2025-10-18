let
  pkgs = import <nixpkgs> {
    config = { };
    overlays = [ ];
  };
in

pkgs.mkShellNoCC {
  packages = with pkgs; [
    protobuf
    python3
  ];
}
