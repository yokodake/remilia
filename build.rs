extern crate target_build_utils;

use std::env;
use std::path::Path;
use std::process::Command;
use target_build_utils::TargetInfo;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

        Command::new("nasm")
                .args(&["kernel/entry.asm", "-felf64", "-o"])
                .arg(&format!("{}/entry.o", out_dir))
                .status()
                .expect("nasm");
        Command::new("rust-ar")
                .args(&["crus", "libentry.a", "entry.o"])
                .current_dir(&Path::new(&out_dir))
                .status()
                .expect("archive");

        println!("cargo:rustc-link-search=native={}", out_dir);
        println!("cargo:rustc-link-lib=static=entry");
}
