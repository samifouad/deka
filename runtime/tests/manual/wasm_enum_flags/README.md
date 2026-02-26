# Wasm enum/flags smoke test

This is a manual runtime check for WIT enum/flags marshalling.

## Steps
1) Build the wasm module:
```sh
deka wasm build @user/enum_flags
```

2) Run the PHPX fixture (make sure the php binary is built):
```sh
cargo build -p php-rs --release
./target/release/php tests/manual/wasm_enum_flags/enum_flags.phpx
```

Expected output includes snake_case enum/flag names, e.g.:
```
mode=read_only
mode2=off
perms=read,write
perms2=read,exec
perms3=read,exec
```
