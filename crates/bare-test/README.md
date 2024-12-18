# Bare Test

A test framework for testing the bare metal.

## Usage

1. Install `ostool`.

    ```shell
    cargo install ostool
    ```

2. setup `.cargo/config.toml`

    ```toml
    [target.'cfg(all(target_os = "none"))']
    runner = "ostool cargo-test"
    [build]
    target = "aarch64-unknown-none"
    ```

3. setup `cargo.toml`.

    ```toml
    [dev-dependencies]
    bare-test = "0.0.1"

    [build-dependencies]
    sparreal-macros = "0.0.1"
    ```

4. setup `build.rs`.

    ```rust
    fn main() {
        sparreal_macros::build_test_setup!();
    }
    ```

5. new `tests` dir and add `tests.rs`.

    ```rust
    #![no_std]
    #![no_main]
    #![feature(custom_test_frameworks)]
    #![test_runner(bare_test::test_runner)]
    #![reexport_test_harness_main = "test_main"]

    extern crate bare_test;

    #[bare_test::entry]
    fn main() {
        test_main();
    }

    use bare_test::println;
    #[test_case]
    fn it_not_works() {
        println!("test2... ");
        assert_eq!(1, 2);
    }
    #[test_case]
    fn it_works1() {
        println!("test1... ");
        assert_eq!(1, 1);
    }
    #[test_case]
    fn test_uart(){
        // map uart data register for using.
        let uart_data_reg = iomap(0x9000000.into(), 0x1000);

        // write to uart, then it will be print to the screen.
        unsafe{
            uart_data_reg.write_volatile(b'A');
            uart_data_reg.write_volatile(b'\n');
        }

        println!("uart test passed!");
    }
    ```

6. run `cargo test --test tests --  --show-output`.
