{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [ clang mold pkg-config protobuf cmake perl rustPlatform.bindgenHook ];
  buildInputs = with pkgs; [ openssl krb5 libxml2 zlib ];
  LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
  LIBGSSAPI_IMPL = "mit";
  SQLX_OFFLINE = "true";
}
