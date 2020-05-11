extern crate bindgen;

use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

fn format_write(builder: bindgen::Builder, output: &str) {
    let s = builder
        .generate()
        .unwrap()
        .to_string()
        .replace("/**", "/*")
        .replace("/*!", "/*");

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(output)
        .unwrap();

    let _ = file.write(s.as_bytes());
}

fn common_builder() -> bindgen::Builder {
    bindgen::builder()
        .raw_line("#![allow(dead_code)]")
        .raw_line("#![allow(non_camel_case_types)]")
        .raw_line("#![allow(non_snake_case)]")
        .raw_line("#![allow(non_upper_case_globals)]")
}

fn find_dir(default: &'static str, env_key: &'static str) -> PathBuf {
    match env::var_os(env_key) {
        Some(val) => PathBuf::from(&val),
        _ => PathBuf::from(default),
    }
}

fn out_dir() -> PathBuf {
    std::env::var("OUT_DIR")
        .expect("OUT_DIR environment var not set.")
        .into()
}

fn main() {
    let cuda_include = find_dir("/opt/cuda/include", "CUDA_INCLUDE_PATH");
    let nvc_include = find_dir(
        "/opt/nvidia-video-codec/include",
        "NVIDIA_VIDEO_CODEC_INCLUDE_PATH",
    );

    // TODO support windows
    println!("cargo:rustc-link-lib=dylib={}", "cuda");
    println!("cargo:rustc-link-lib=dylib={}", "nvcuvid");
    println!("cargo:rustc-link-lib=dylib={}", "nvidia-encode");

    cc::Build::new()
        // .compiler("/usr/bin/clang++")
        .cpp(true)
        .cuda(true)
        //.cpp_link_stdlib("stdc++")
        .file("src/NvEncoderCuda.cpp")
        .include(&cuda_include)
        .include(&nvc_include)
        .out_dir(&out_dir())
        .compile("nv_encoder_cuda");

    println!("cargo:rustc-link-lib=static=nv_encoder_cuda");
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=dylib=c++");
    } else {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }

    let cuda_builder = common_builder()
        .clang_arg(format!("-I{}", cuda_include.to_string_lossy()))
        .header(cuda_include.join("cuda.h").to_string_lossy());

    // Manually fix the comment so rustdoc won't try to pick them
    format_write(cuda_builder, "src/cuda.rs");

    let cuvid_builder = common_builder()
        .clang_arg(format!("-I{}", nvc_include.to_string_lossy()))
        .clang_arg(format!("-I{}", cuda_include.to_string_lossy()))
        .header(nvc_include.join("nvcuvid.h").to_string_lossy());

    format_write(cuvid_builder, "src/cuvid.rs");

    let nvenc_builder = common_builder()
        .clang_arg(format!("-I{}", nvc_include.to_string_lossy()))
        .header(nvc_include.join("nvEncodeAPI.h").to_string_lossy());
    format_write(nvenc_builder, "src/nvenc.rs");

    // if cfg!(target_os = "macos") {
    //     println!("cargo:rustc-link-lib=dylib=c++");
    // } else {
    println!("cargo:rustc-link-lib=dylib=stdc++");
    // }
    let nvenccuda_builder = common_builder()
        .clang_arg(format!("-I{}", nvc_include.to_string_lossy()))
        .clang_arg(format!("-I{}", cuda_include.to_string_lossy()))
        .clang_args(&["-x", "c++", "-std=c++14"])
        //.clang_arg("-std=gnu++11")
        .clang_arg("-stdlib=libstdc++")
        .whitelist_type("NvEncoderCuda")
        .header("src/NvEncoderCuda.hpp");
    format_write(nvenccuda_builder, "src/nvenc_cuda.rs");
}
