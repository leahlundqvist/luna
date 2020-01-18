cargo xbuild --target x86_64-luna.json
cargo bootimage --target x86_64-luna.json
qemu-system-x86_64 -drive format=raw,file=target/x86_64-luna/debug/bootimage-luna.bin
