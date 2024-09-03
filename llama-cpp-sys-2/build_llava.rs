//! # llava related build script
//! ## Changes in the build script
//! * `compile_bindings`: add llava related files, and prefix of type and function
//! * replace `push_feature_flags`
//! * call `compile_llava`
//! ```ignore
//!     let llava_cxx = cxx.clone();
//!     build_llava::compile_llava(llava_cxx);
//! ```
//! 

use cc::Build;

use crate::LLAMA_PATH;

/// true if at least one force feature is turned on
const FORCE_ARCH: bool = cfg!(any(
    feature = "force_avx",
    feature = "force_avx2",
    feature = "force_avx512f",
    feature = "force_avx512bw",
    feature = "force_avx512vbmi",
    feature = "force_avx512vnni"
));

/// Add platform appropriate flags and definitions based on enabled features.
/// * By default, build script will detect the CPU arch features and push the flags
/// * If any feature `force_*` is set manually, then force to push the flags
pub fn push_feature_flags(cx: &mut Build, cxx: &mut Build) {
    // TODO in llama.cpp's cmake (https://github.com/ggerganov/llama.cpp/blob/9ecdd12e95aee20d6dfaf5f5a0f0ce5ac1fb2747/CMakeLists.txt#L659), they include SIMD instructions manually, however it doesn't seem to be necessary for VS2022's MSVC, check when it is needed

    if cfg!(any(target_arch = "x86", target_arch = "x86_64")) {
        if cfg!(feature = "native") && cfg!(target_os = "linux") {
            cx.flag("-march=native");
            cxx.flag("-march=native");
        }

        if cfg!(target_feature = "fma") && cfg!(target_family = "unix") {
            cx.flag("-mfma");
            cxx.flag("-mfma");
        }

        if cfg!(target_feature = "f16c") && cfg!(target_family = "unix") {
            cx.flag("-mf16c");
            cxx.flag("-mf16c");
        }

        if cfg!(target_family = "unix") {
            if (!FORCE_ARCH && is_x86_feature_detected!("avx512f"))
                || cfg!(feature = "force_avx512f")
            {
                cx.flag("-mavx512f");
                cxx.flag("-mavx512f");

                if (!FORCE_ARCH && is_x86_feature_detected!("avx512bw"))
                    || cfg!(feature = "force_avx512bw")
                {
                    cx.flag("-mavx512bw");
                    cxx.flag("-mavx512bw");
                }

                if (!FORCE_ARCH && is_x86_feature_detected!("avx512vbmi"))
                    || cfg!(feature = "force_avx512vbmi")
                {
                    cx.flag("-mavx512vbmi");
                    cxx.flag("-mavx512vbmi");
                }

                if (!FORCE_ARCH && is_x86_feature_detected!("avx512vnni"))
                    || cfg!(feature = "force_avx512vnni")
                {
                    cx.flag("-mavx512vnni");
                    cxx.flag("-mavx512vnni");
                }
            }

            if (!FORCE_ARCH && is_x86_feature_detected!("avx2")) || cfg!(feature = "force_avx2") {
                cx.flag("-mavx2");
                cxx.flag("-mavx2");
            }

            if (!FORCE_ARCH && is_x86_feature_detected!("avx")) || cfg!(feature = "force_avx") {
                cx.flag("-mavx");
                cxx.flag("-mavx");
            }
        } else if cfg!(target_family = "windows") {
            if (!FORCE_ARCH && is_x86_feature_detected!("avx512f"))
                || cfg!(feature = "force_avx512f")
            {
                cx.flag("/arch:AVX512");
                cxx.flag("/arch:AVX512");

                if (!FORCE_ARCH && is_x86_feature_detected!("avx512vbmi"))
                    || cfg!(feature = "force_avx512vbmi")
                {
                    cx.define("__AVX512VBMI__", None);
                    cxx.define("__AVX512VBMI__", None);
                }

                if (!FORCE_ARCH && is_x86_feature_detected!("avx512vnni"))
                    || cfg!(feature = "force_avx512vnni")
                {
                    cx.define("__AVX512VNNI__", None);
                    cxx.define("__AVX512VNNI__", None);
                }
            } else if (!FORCE_ARCH && is_x86_feature_detected!("avx2"))
                || cfg!(feature = "force_avx2")
            {
                cx.flag("/arch:AVX2");
                cxx.flag("/arch:AVX2");
            } else if (!FORCE_ARCH && is_x86_feature_detected!("avx"))
                || cfg!(feature = "force_avx")
            {
                cx.flag("/arch:AVX");
                cxx.flag("/arch:AVX");
            }
        }
    }
}


pub fn compile_llava(mut cxx: Build) {
    println!("Compiling Llama.cpp..");
    let llama_include = LLAMA_PATH.join("include");
    let ggml_include = LLAMA_PATH.join("ggml").join("include");
    let common_dir = LLAMA_PATH.join("common");
    let llava_dir = LLAMA_PATH.join("examples").join("llava");
    cxx.std("c++11")
        .include(llava_dir.clone())
        .include(common_dir.clone())
        .include(llama_include)
        .include(ggml_include)
        .file(llava_dir.join("llava.cpp"))
        .file(llava_dir.join("clip.cpp"))
        .file("src/llava_sampling.cpp")
        .file("src/build-info.cpp")
        .file(common_dir.join("sampling.cpp"))
        .file(common_dir.join("grammar-parser.cpp"))
        .file(common_dir.join("json-schema-to-grammar.cpp"))
        .file(common_dir.join("common.cpp"))
        .compile("llava");
}
