<p align="center">
    <a href="https://developerdao.com">
    <img src="https://raw.githubusercontent.com/Developer-DAO/developerdao.com/main/public/logo512.png" alt="logo" width="80" height="80"/>
    </a>
    <h2 align="center">FLS</h2>
    <p align="center">
    <a href="https://github.com/ethereum/fe">Fe</a> Language Server
    </p>
</p>

The `FLS` is designed to be frontend-independent. We hope it will be widely adopted by different editors and IDEs.

## Setup

### Step 1: Install rustup

You can install [rustup](http://rustup.rs/) on many platforms. This will help us quickly install the FLS and its dependencies.

If you already have rustup installed, update to ensure you have the latest
rustup and compiler:

```
rustup update
```

### Step 2: Install the FLS

Once you have rustup installed, run the following commands:

```
cargo install --git https://github.com/Developer-DAO/fls.git
```

## Running

The FLS is built to work with many IDEs and editors, we mostly use
VSCode to test the FLS. The easiest way is to use the [published extension](https://github.com/Developer-DAO/vscode-fe).

You'll know it's working when you see this in the status bar at the bottom, with
a spinning indicator:

`FLS: working ◐`

Once you see:

`FLS`

Then you have the full set of capabilities available to you.  You can goto def, find all refs, rename, goto type, etc.
Completions are also available.  As you type, your code will be checked and error squiggles will be reported when errors occur.
You can hover these squiggles to see the text of the error.

## License

[Apache 2.0](https://opensource.org/licenses/Apache-2.0)
