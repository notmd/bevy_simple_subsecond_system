# Bevy Simple Subsecond System

[![crates.io](https://img.shields.io/crates/v/bevy_simple_subsecond_system)](https://crates.io/crates/bevy_simple_subsecond_system)
[![docs.rs](https://docs.rs/bevy_simple_subsecond_system/badge.svg)](https://docs.rs/bevy_simple_subsecond_system)


Hotpatch your Bevy systems, allowing you to change their code while the app is running and directly seeing the results!
This is a intermediate solution you can use until [Bevy has implement this feature upstream](https://github.com/bevyengine/bevy/issues/19296).

Powered by [Dioxus' subsecond](https://github.com/DioxusLabs/dioxus/releases/tag/v0.7.0-alpha.0#rust-hot-patching)  
Please report all hotpatch-related problems to them :)

> ⚠️ *Should* work on Windows *somehow*, but I haven't yet figured out how! Let me know if you made it!


## First Time Installation

First, we'll install [cargo-binstall](https://github.com/cargo-bins/cargo-binstall). It's not strictly required, but it will make the setup much quicker.
Click your OS below on instructions for how to do this
<details>
<summary>
Windows
</summary>

```pwsh
Set-ExecutionPolicy Unrestricted -Scope Process; iex (iwr "https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.ps1").Content
```

</details>

<details>
<summary>
macOS
</summary>

```sh
brew install cargo-binstall
```
or, if you don't use `brew`, same as on Linux.
</details>

<details>
<summary>
Linux
</summary>

```sh
curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
```
</details>
<details>
<summary>
Build from source
</summary>

```sh
cargo install cargo-binstall
```
</details>

Now, we need to install the Dioxus CLI of the newest alpha build:
```sh
cargo binstall dioxus-cli@0.7.0-alpha.0
```


Then make sure you're not using [LD as your linker](https://github.com/DioxusLabs/dioxus/issues/4144).
Click your OS below on instructions for how to do this

> In case you already configured a linker, setting `rustflags = ["-C", "link-arg=-fuse-ld=/path/to/your/linker"]` is surprisingly [not enough](https://github.com/DioxusLabs/dioxus/issues/4146)!

<details>
<summary>
Windows
</summary>

Sorry friend, I didn't test this. All I know is that you can install `rust-lld.exe` by running

```sh
cargo binstall cargo-binutils
rustup component add llvm-tools-preview
```

</details>

<details>
<summary>
macOS
</summary>

You're in luck! The default linker on macOS is already something other than LD. You don't have to change a thing :)
</details>

<details>
<summary>
Linux
</summary>

Download `clang` and [`mold`](https://github.com/rui314/mold) for your distribution, e.g.
```sh
sudo apt-get install clang mold
```
Then, replace your system `ld` with a symlink to `mold`. The most brutal way to do this is:
```sh
cd /usr/bin
sudo mv ld ld-real
sudo ln -s mold ld
```

</details>

## Usage

Add the crate to your dependencies:

```sh
cargo add bevy_simple_subsecond_system
```
Then add the plugin to your app:

```rust,ignore
use bevy::prelude::*;
use bevy_simple_subsecond_system::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SimpleSubsecondPlugin::default())
        // rest of the setup
        .run();
}

```

Now you can annotate your systems with `#[hot]` to enable hotpatching for them:

```rust,ignore
use bevy_simple_subsecond_system::prelude::*;

#[hot]
fn greet() {
    info!("Hello from a hotpatched system! Try changing this string while the app is running!")
}
```
Note that `greet` is a regular Bevy system, so use whatever parameters you'd like.

After adding the system to your app, run it with

```sh
dx serve --hot-patch
```

Now try changing the string and saving the file *while the app is running*. If all goes well, it should print your new string!

<details>
<summary>Full code</summary>

```rust,ignore
use bevy::prelude::*;
use bevy_simple_subsecond_system::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SimpleSubsecondPlugin::default())
        .add_systems(Update, greet)
        .run();
}

#[hot]
fn greet() {
    info!("Hello from a hotpatched system! Try changing this string while the app is running!")
}
```

</details>

## Examples


Run the examples with
```sh
dx serve --hot-patch --package name_of_the_example
```

e.g.
```sh
dx serve --hot-patch --package ui
```

## Known Limitations

- Cannot combine mold as your Rust linker with a global target dir: https://github.com/DioxusLabs/dioxus/issues/4149
- Attaching a debugger is problaby not going to work. Let me know if you try!


## Compatibility

| bevy        | bevy_simple_subsecond_system |
|-------------|------------------------|
| 0.16        | 0.1                    |
