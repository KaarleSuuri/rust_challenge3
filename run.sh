cargo build --release
riscv64-unknown-elf-objcopy -O binary target/riscv32imac-unknown-none-elf/release/challenge3 firmware.bin
dfu-util -a 0 -s 0x08000000:leave -D firmware.bin
