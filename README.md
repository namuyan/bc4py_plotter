bc4py poc plotter
====
This is PoC hash plot tool.

Requirements
----
* Linux/Windows/etc
* Rust **nightly**
* main memory 2GB

How to use
----
```bash
git clone https://github.com/namuyan/bc4py_plotter
cd bc4py_plotter
cargo build --release
cp target/release/bc4py_plotter .
./bc4py_plotter
```

Rust manual install
----
[Install document](https://forge.rust-lang.org/other-installation-methods.html#more-rustup)

or

`curl https://sh.rustup.rs -sSf | sh`
```text
Current installation options:
 
   default host triple: x86_64-unknown-linux-gnu
     default toolchain: stable
  modify PATH variable: yes
 
1) Proceed with installation (default)
2) Customize installation
3) Cancel installation
```
you need to select **2) Customize installation**

```text
Default host triple?
```
push **Enter** [What?](https://stackoverflow.com/questions/49368232/what-is-a-default-host-triple-in-rust)

```text
Default toolchain? (stable/beta/nightly/none)
```
you need to select **nightly**

```text
Modify PATH variable? (y/n)
```
you need to select **y**

Trouble shooting
----
* `failed to run custom build command for 'openssl-sys vX.X.X'` => `sudo apt install libssl-dev`
* consume too memory => please edit **MAX_MEMORY_SIZE** of cli_tool.rs

Licence
----
MIT

Author
----
[namuyan_mine](http://twitter.com/namuyan_mine)
