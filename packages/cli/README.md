# Compilation

```sh
cargo build --release
```

Output will be located at the root of the repo in **target/release/cli**

# Unit testing

Run tests with code coverage :

```sh
cargo tarpaulin --out html --output-dir packages/cli/coverage --exclude-files "**/target/**/*" "**/tarpaulin_temp/**/*"
```

Code coverage located in **coverage/**

If you are in dev mode you might want to skip recompilation everytime you run unit tests, this can be done the following way :

```
cargo tarpaulin --out html --output-dir packages/cli/coverage --exclude-files "**/target/**/*" "**/tarpaulin_temp/**/*" --target-dir tarpaulin_temp --skip-clean
```

# Documentation

Generate documentation :

```
cargo doc --no-deps
```
