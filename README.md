bc4py poc plotter
====
This is PoC hash plot tool.

Requirements
====
* Linux/Windows/etc
* Rust **nightly** [Install](https://forge.rust-lang.org/other-installation-methods.html#more-rustup) or `curl https://sh.rustup.rs -sSf | sh`

How to use
====
```bash
git clone https://github.com/namuyan/bc4py_plotter
cd bc4py_plotter
cargo build --release
cp target/release/bc4py_plotter .
./bc4py_plotter
```

Rust manual install
====
```text
info: downloading installer
 
Welcome to Rust!
 
This will download and install the official compiler for the Rust programming 
language, and its package manager, Cargo.
 
It will add the cargo, rustc, rustup and other commands to Cargo's bin 
directory, located at:
 
  /home/account/.cargo/bin
 
This path will then be added to your PATH environment variable by modifying the
profile file located at:
 
  /home/account/.profile
 
You can uninstall at any time with rustup self uninstall and these changes will
be reverted.
 
Current installation options:
 
   default host triple: x86_64-unknown-linux-gnu
     default toolchain: stable
  modify PATH variable: yes
 
1) Proceed with installation (default)
2) Customize installation
3) Cancel installation
>2
 
I'm going to ask you the value of each these installation options.
You may simply press the Enter key to leave unchanged.
 
Default host triple?
 
 
Default toolchain? (stable/beta/nightly/none)
nightly
 
Modify PATH variable? (y/n)
y
 
 
Current installation options:
 
   default host triple: x86_64-unknown-linux-gnu
     default toolchain: nightly
  modify PATH variable: yes
 
1) Proceed with installation (default)
2) Customize installation
3) Cancel installation
>1
 
info: syncing channel updates for 'nightly-x86_64-unknown-linux-gnu'
info: latest update on 2019-02-22, rust version 1.34.0-nightly (633d75ac1 2019-02-21)
info: downloading component 'rustc'
 85.3 MiB /  85.3 MiB (100 %)   1.9 MiB/s ETA:   0 s                
info: downloading component 'rust-std'
 55.8 MiB /  55.8 MiB (100 %)   1.8 MiB/s ETA:   0 s                
info: downloading component 'cargo'
  4.3 MiB /   4.3 MiB (100 %)   1.8 MiB/s ETA:   0 s                
info: downloading component 'rust-docs'
 10.2 MiB /  10.2 MiB (100 %)   2.1 MiB/s ETA:   0 s                
info: installing component 'rustc'
info: installing component 'rust-std'
info: installing component 'cargo'
info: installing component 'rust-docs'
info: default toolchain set to 'nightly'
 
  nightly installed - rustc 1.34.0-nightly (633d75ac1 2019-02-21)
 
 
Rust is installed now. Great!
 
To get started you need Cargo's bin directory ($HOME/.cargo/bin) in your PATH 
environment variable. Next time you log in this will be done automatically.
 
To configure your current shell run source $HOME/.cargo/env
```

Trouble shooting
====
* `failed to run custom build command for 'openssl-sys vX.X.X'` => `sudo apt install libssl-dev`

Licence
====
MIT

Author
====
[namuyan_mine](http://twitter.com/namuyan_mine)
