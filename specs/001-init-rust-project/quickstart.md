# Quickstart: Project Initialization

This guide explains how to create the initial Rust project structure.

## Prerequisites

- Rust toolchain (rustc and cargo). You can install it from [rust-lang.org](https://www.rust-lang.org/tools/install).

## Steps

1.  **Navigate to the desired parent directory.**
    Open your terminal and `cd` into the directory where you want to create the project.

2.  **Run the initialization command.**
    ```bash
    cargo init --bin rsudp-rust
    ```
    This will create a new directory named `rsudp-rust` containing the project skeleton.

3.  **Enter the project directory.**
    ```bash
    cd rsudp-rust
    ```

4.  **Build the project.**
    ```bash
    cargo build
    ```
    This command compiles the project. You should see it finish without errors.

5.  **Run the application.**
    ```bash
    cargo run
    ```

## Expected Outcome

After running `cargo run`, you should see the following output in your terminal:

```
Hello, world!
```
