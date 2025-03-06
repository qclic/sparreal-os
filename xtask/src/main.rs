#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

#[cfg(not(target_os = "none"))]
mod cli;

#[cfg(not(target_os = "none"))]
fn main() {
    cli::main();
}
