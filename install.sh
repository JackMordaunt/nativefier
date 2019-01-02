cargo build --release 
upx target/release/nativefier 
cp target/release/nativefier ~/.cargo/bin/nativefier