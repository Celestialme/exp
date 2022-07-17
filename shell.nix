{ pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell {
    
    nativeBuildInputs = [ 
      pkgs.gcc
      pkgs.rustc
      pkgs.rustfmt
      pkgs.cargo
      pkgs.cargo-edit
      pkgs.rust-analyzer
      pkgs.dbus
      pkgs.pkgconfig
      pkgs.openssl
      pkgs.sass
      pkgs.glib
      pkgs.cairo
      pkgs.pango
      pkgs.atk
      pkgs.gdk-pixbuf
      pkgs.libsoup
      pkgs.librsvg
      pkgs.patchelf

     ];
     buildInputs = [ 
      pkgs.cargo
     ];
     shellHook = ''
     cargo build --release
  '';
}
