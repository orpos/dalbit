# Dal
Dal(달) is a Luau-to-Lua transpiler based on `darklua`, designed specifically for `Lua 5.3`.

## Note
This project is still in W.I.P

## TO-DOs
- [x] Implement CLI.
- [x] Implement basic transpilation process using `darklua` and `full-moon`.
- [ ] Implement modifiers (such as converting number literals and generalized iterations)
- [ ] Implement basic lua polyfills.

## Installation
Coming soon (will be available at `rokit` and `crates.io`(for `cargo install`))

## Usage

### `init`
Initializes dal manifest file in the current path.
```sh
dal init
```

### `fetch`
Fetches and updates lua polyfills.
* This polyfill can be found [here](https://github.com/CavefulGames/dal-polyfill).
```sh
dal fetch
```

### `transpile`
Transpiles luau code to lua code.
```sh
dal transpile [input] [output]
```

## Special Thanks
- [seaofvoices/darklua](https://github.com/seaofvoices/darklua) - Providing important and cool lua mutating rules.
- [Kampfkarren/full-moon](https://github.com/Kampfkarren/full-moon) - A lossless Lua parser.

## Trivia
The name of this project, Dal, translates to "moon" in Korean.
