# Bevy Simple Subsecond System

[![crates.io](https://img.shields.io/crates/v/bevy_simple_subsecond_system)](https://crates.io/crates/bevy_simple_subsecond_system)
[![docs.rs](https://docs.rs/bevy_simple_subsecond_system/badge.svg)](https://docs.rs/bevy_simple_subsecond_system)


Hotpatch your Bevy systems, allowing you to change their code while the app is running and directly seeing the results!
This is a intermediate solution you can use until [Bevy has implement this feature upstream](https://github.com/bevyengine/bevy/issues/19296).

Powered by [Dioxus' subsecond](https://github.com/DioxusLabs/dioxus/releases/tag/v0.7.0-alpha.0#rust-hot-patching)  
Please report all hotpatch-related problems to them :)


<https://github.com/user-attachments/assets/a44e446b-b2bb-4e10-81c3-3f20cccadea0>



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

<details><summary>Windows</summary>

```sh
cargo install dioxus-cli --git https://github.com/DioxusLabs/dioxus/ --branch jk/fix-dyn-link-subsecond
```

</details>

<details><summary>Non-Windows</summary>

```sh
cargo binstall dioxus-cli@0.7.0-alpha.0
```

</details>
<br/>

Then make sure you're not using [LD as your linker](https://github.com/DioxusLabs/dioxus/issues/4144).
Click your OS below on instructions for how to do this

> In case you already configured a linker, setting `rustflags = ["-C", "link-arg=-fuse-ld=/path/to/your/linker"]` is surprisingly [not enough](https://github.com/DioxusLabs/dioxus/issues/4146)!

<details>
<summary>
Windows
</summary>

The linker should work out of the box, but there may be issues with path lengths.

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

On NixOS you can do this in a shell by replacing:
```nix
pkgs.mkShell {
    # ..
}
```
with:
```nix
pkgs.mkShell.override {
    stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.clangStdenv;
} {
    # ..
}
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
dx serve --hot-patch --example name_of_the_example
```

e.g.
```sh
dx serve --hot-patch --example rerun_setup
```



## Advanced Usage
There are some more things you can hot-patch, but they come with extra caveats right now

<details>
<summary>Limitations when using these features</summary>

- Annotating a function relying on local state will clear it every frame. Notably, this means you should not use `#[hot]` on a system that uses any of the following:
  - `EventReader`
  - `Local`
  - Queries filtering with `Added`, `Changed`, or `Spawned`
- Some signatures are not supported, see the tests. Some have `#[hot]` commented out to indicate this
- All hotpatched systems run as exclusive systems, meaning they won't run in parallel
</details>


<details>
<summary>
<sig>Setup Methods</sig>
</summary>

UI is often spawned in `Startup` or `OnEnter` schedules. Hot-patching such setup systems would be fairly useless, as they wouldn't run again.
For this reason, the plugin supports automatically rerunning systems that have been hot-patched. To opt-in, replace `#[hot]` with `#[hot(rerun_on_hot_patch = true)]`.
See the `rerun_setup` example for detailed instructions.

</details>

<details>
<summary>
<sig>Change signatures at runtime</sig>
</summary>

Replace `#[hot]` with `#[hot(hot_patch_signature = true)]` to allow changing a system's signature at runtime.
This allows you to e.g. add additional `Query` or `Res` parameters or modify existing ones.
</details>


## Features

- Change systems' code and see the effect live at runtime
- Change system signatures at runtime, e.g. by adding a new query or modifying an existing one
- If your system calls other functions, you can also change those functions' code at runtime
- Rerun setup systems automatically when changing them
- Extremely small API: You only need the plugin struct and the `#[hot]` attribute
- Automatically compiles itself out on release builds and when targetting Wasm. The `#[hot]` attribute does simply nothing on such builds.

## Known Limitations

- Cannot [combine mold as your Rust linker with a global target dir](https://github.com/DioxusLabs/dioxus/issues/4149)
- Using this [breaks dynamic linking](https://github.com/DioxusLabs/dioxus/issues/4154)
- A change in the definition of structs that appear in hot-patched systems at runtime will result in your query failing to match, as that new type does not exist in `World` yet.
  - Practically speaking, this means you should not change the definition of `Resource`s and `Component`s of your system at runtime
- Only [the topmost binary is hotpatched](https://github.com/DioxusLabs/dioxus/issues/4160), meaning your app is not allowed to have a `lib.rs` or a workspace setup.
- Attaching a debugger is problaby not going to work. Let me know if you try!
- I did not test all possible ways in which systems can be used. Does piping work? Does `bevy_mod_debugdump` still work? Maybe. Let me know!
- Only functions that exist when the app is launched are considered while hotpatching. This means that if you have a system `A` that calls a function `B`, 
  changing `B` will only work at runtime if that function existed already when the app was launched.
- Does nothing on Wasm. This is not a technical limitation, just something we didn't implement yet.


## Compatibility

| bevy        | bevy_simple_subsecond_system |
|-------------|------------------------|
| 0.16        | 0.1                    |
