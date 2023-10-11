# Statemachine

## API
```bash
cargo run --bin web_api
```

## UI
```bash
# run server
cargo run --bin web_ui
```

### Bundle Setup
```bash
# setup rollup
npm i rollup @rollup/plugin-node-resolve
echo 'PATH="$HOME/node_modules/.bin/:$PATH' > ~/.profile

# install deps
npm i codemirror @codemirror/state @codemirror/lang-json @codemirror/autocomplete
```
### Bundle Build
```bash
# build bundle
rollup web_ui/src/editor.mjs -f iife -o web_ui/static/editor.bundle.js -p @rollup/plugin-node-resolve -p @rollup/plugin-commonjs -p @rollup/plugin-json --name editorBundle
```

## Statemachine
```bash
cargo run --bin statemachine
```

## TODO
```bash
cargo run --bin emulator
```