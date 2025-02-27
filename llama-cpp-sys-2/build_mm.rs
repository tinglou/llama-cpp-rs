use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::Context;
use cmake::Config;

macro_rules! debug_log {
    ($($arg:tt)*) => {
        if std::env::var("BUILD_DEBUG").is_ok() {
            println!("cargo:warning=[DEBUG] {}", format!($($arg)*));
        }
    };
}

/// cmake build only once
/// build.rs:384
/// ```ignore
/// let build_dir = config.build();
/// let build_dir = build_mm::cmake_build(&mut config);
/// ```
pub fn cmake_build(config: &mut Config) -> PathBuf {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let build_dir = out_dir.join("build");

    //  check if cmake cache exists
    let cmake_cache = build_dir.join("CMakeCache.txt");
    if cmake_cache.exists() {
        debug_log!("cmake cache exists, skip cmake build");
        // build_dir returned by `cmake::Congfig::build` is the same as oput_dir by default
        out_dir
    } else {
        let dir = config.build();
        debug_log!("build dir: {}", dir.display());
        dir
    }
}

/// pre cmake build, called before cmake build
/// build.rs:382
pub fn pre_cmake_build(config: &mut Config) -> anyhow::Result<()> {
    let target = env::var("TARGET")?;

    if cfg!(windows) {
        // Ninja is faster than Makefile/msbuild on Windows
        // config.generator("Ninja");
    }

    if cfg!(windows) && !cfg!(debug_assertions) {
        // release
        debug_log!("disabe log in release mode");
        config.cflag("/D NDEBUG");
        config.cflag("/D LLAVA_LOG_OFF");
        config.cxxflag("/D NDEBUG");
        config.cxxflag("/D LLAVA_LOG_OFF");
    }

    // 1. turn on examples to enable llava
    config.define("LLAMA_BUILD_EXAMPLES", "ON");

    // 2. turn off metal and openmp on macOS x86_64
    if cfg!(target_os = "macos") && target.contains("x86_64") {
        // macOS x86_64 doesn't support OpenMP and Metal
        config.define("GGML_OPENMP", "OFF");
        config.define("GGML_METAL", "OFF");
        config.define("GGML_METAL_EMBED_LIBRARY", "OFF");
    }

    if cfg!(feature = "kompute") {
        config.define("GGML_KOMPUTE", "ON");
    }

    if cfg!(feature = "rpc") {
        config.define("GGML_RPC", "ON");
    }

    if cfg!(feature = "opencl") {
        config.define("GGML_OPENCL", "ON");
    }

    // 3. turn on sycl on windows, see [sycl](llama-cpp-sys-2\\llama.cpp\\docs\\backend\\SYCL.md)
    if cfg!(windows) && (cfg!(feature = "sycl-f16") || cfg!(feature = "sycl-f32")) {
        // 只有编译 sycl 时，采用 Ninja
        config.generator("Ninja");

        // config.very_verbose(true);
        if cfg!(feature = "sycl-f16") && cfg!(feature = "sycl-f32") {
            panic!("cannot enable both sycl-f16 and sycl-f32");
        }

        // `/EHsc`` 是一个编译器选项，用于指定编译器生成的异常处理模型。这个选项是在使用Microsoft Visual C++编译器
        // （如MSVC）时使用的，它告诉编译器启用C++的异常处理模型，并遵循C++标准来处理异常。
        // 具体来说，/EHsc 选项的含义包括：
        //
        // - **启用标准C++堆栈展开**：当异常发生时，编译器会生成代码来展开（unwind）堆栈，销毁在异常抛出点和捕获点之间创建的
        // 自动存储对象（如局部变量），并回收其资源。这个过程是C++异常处理的核心部分，确保了程序的健壮性和资源的正确管理。
        //
        // - **捕获标准C++异常**：使用catch(...)语法时，编译器会捕获并处理标准C++异常。这意味着，如果代码中抛出了一个C++异常，
        // 并且有一个相应的catch块来捕获它，那么该异常将被正确处理。
        //
        // - **对extern "C"函数的异常处理假设**：除非另外指定/EHc，否则编译器假定声明为extern "C"的函数可能抛出C++异常。
        // 这是为了确保在混合使用C和C++代码时，异常能够被正确处理。如果使用了/EHc选项，并且与/EHs（或/EHsc）一起使用，
        // 编译器将假定extern "C"函数不会抛出C++异常。
        config.cxxflag("/EHsc");
        // `/W3` 设置编译器的警告级别为3。警告是编译器用来通知开发者代码中可能存在的问题或不符合最佳实践的地方，尽管编译器不会强制要求修复这些警告
        config.cxxflag("/W3");
        // `/GR`` 启用运行时类型信息（RTTI）的支持。RTTI是C++语言的一个特性，它允许程序在运行时识别对象的类型。
        config.cxxflag("/GR");

        config.define("GGML_SYCL", "ON");
        config.define("CMAKE_C_COMPILER", "cl");
        config.define("CMAKE_CXX_COMPILER", "icx");
        if cfg!(feature = "sycl-f16") {
            config.define("GGML_SYCL_F16", "ON");
        }
    }

    // 4. cpu features
    if cfg!(feature = "ggml-cpu-hbm") {
        config.define("GGML_CPU_HBM", "ON");
    }
    if cfg!(feature = "ggml-cpu-aarch64") {
        config.define("GGML_CPU_AARCH64", "ON");
    }
    if cfg!(feature = "ggml-avx") {
        config.define("GGML_AVX", "ON");
    }
    if cfg!(feature = "ggml-avx-vnni") {
        config.define("GGML_AVX_VNNI", "ON");
    }
    if cfg!(feature = "ggml-avx2") {
        config.define("GGML_AVX2", "ON");
    }
    if cfg!(feature = "ggml-avx512") {
        config.define("GGML_AVX512", "ON");
    }
    if cfg!(feature = "ggml-avx512-vbmi") {
        config.define("GGML_AVX512_VBMI", "ON");
    }
    if cfg!(feature = "ggml-avx512-vnni") {
        config.define("GGML_AVX512_VNNI", "ON");
    }
    if cfg!(feature = "ggml-avx512-bf16") {
        config.define("GGML_AVX512_BF16", "ON");
    }
    if cfg!(feature = "ggml-fma") {
        config.define("GGML_FMA", "ON");
    }
    if cfg!(feature = "ggml-f16c") {
        config.define("GGML_F16C", "ON");
    }
    if cfg!(feature = "ggml-amx-tile") {
        config.define("GGML_AMX_TILE", "ON");
    }
    if cfg!(feature = "ggml-amx-int8") {
        config.define("GGML_AMX_INT8", "ON");
    }
    if cfg!(feature = "ggml-amx-bf16") {
        config.define("GGML_AMX_BF16", "ON");
    }
    if cfg!(feature = "ggml-lasx") {
        config.define("GGML_LASX", "ON");
    }
    if cfg!(feature = "ggml-lsx") {
        config.define("GGML_LSX", "ON");
    }
    if cfg!(feature = "ggml-rvv") {
        config.define("GGML_RVV", "ON");
    }

    Ok(())
}

/// post cmake build, called in the end of `main`
/// ```
pub fn post_cmake_build(out_dir: &Path, build_shared_libs: bool) -> anyhow::Result<()> {
    if cfg!(windows) && (cfg!(feature = "sycl-f16") || cfg!(feature = "sycl-f32")) {
        copy_sycl_libs(out_dir, build_shared_libs)?;
    } else {
        copy_llava_libs(out_dir, build_shared_libs)?;
    }

    // cheat build.rs:386
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("Failed to get CARGO_MANIFEST_DIR");
    let llama_src = Path::new(&manifest_dir).join("llama.cpp");
    let build_info_src = llama_src.join("common/build-info.cpp");
    let build_info_target = out_dir.join("build-info.cpp");
    safe_hard_link(build_info_target, build_info_src)?;
    Ok(())
}

/// check if src file is newer than dst file, if yes, hard link src to dst
fn safe_hard_link<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> anyhow::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    let src_metadata = std::fs::metadata(src);
    let dst_metadata = std::fs::metadata(dst);

    match (src_metadata, dst_metadata) {
        (Ok(src_md), Ok(dst_md)) => {
            // 可以根据需要添加更多的元数据检查
            if src_md.len() != dst_md.len() || src_md.modified()? != dst_md.modified()? {
                std::fs::remove_file(dst)?;
                std::fs::hard_link(src, dst)?;
            }
        }
        (Ok(_), Err(_)) => {
            std::fs::hard_link(src, dst)?;
        }
        (Err(_), _) => {
            anyhow::bail!(
                "src file not found, src: {}, dst: {}",
                src.display(),
                dst.display()
            );
        }
    }
    Ok(())
}

/// Copy sycl libs to out_dir
/// works only on windows with oneAPI 2025.0
fn copy_sycl_libs(_out_dir: &Path, build_shared_libs: bool) -> Result<(), anyhow::Error> {
    let libs = vec![
        "C:\\Program Files (x86)\\Intel\\oneAPI\\compiler\\latest\\bin\\libmmd.dll",
        "C:\\Program Files (x86)\\Intel\\oneAPI\\compiler\\latest\\bin\\svml_dispmd.dll",
        "C:\\Program Files (x86)\\Intel\\oneAPI\\compiler\\latest\\bin\\sycl8.dll",
        "C:\\Program Files (x86)\\Intel\\oneAPI\\compiler\\latest\\bin\\ur_win_proxy_loader.dll",
        "C:\\Program Files (x86)\\Intel\\oneAPI\\compiler\\latest\\bin\\libiomp5md.dll",
        "C:\\Program Files (x86)\\Intel\\oneAPI\\compiler\\latest\\bin\\ur_loader.dll",
        "C:\\Program Files (x86)\\Intel\\oneAPI\\compiler\\latest\\bin\\ur_adapter_opencl.dll",
        "C:\\Program Files (x86)\\Intel\\oneAPI\\compiler\\latest\\bin\\ur_adapter_level_zero.dll",
        "C:\\Program Files (x86)\\Intel\\oneAPI\\compiler\\latest\\bin\\intelocl64.dll",
        "C:\\Program Files (x86)\\Intel\\oneAPI\\dnnl\\latest\\bin\\dnnl.dll",
        "C:\\Program Files (x86)\\Intel\\oneAPI\\mkl\\latest\\bin\\mkl_sycl_blas.5.dll",
        "C:\\Program Files (x86)\\Intel\\oneAPI\\mkl\\latest\\bin\\mkl_tbb_thread.2.dll",
        "C:\\Program Files (x86)\\Intel\\oneAPI\\mkl\\latest\\bin\\mkl_core.2.dll",
        "C:\\Program Files (x86)\\Intel\\oneAPI\\tbb\\latest\\bin\\tbb12.dll",
        "C:\\Program Files (x86)\\Intel\\oneAPI\\tbb\\latest\\bin\\tbbmalloc.dll",
        "C:\\Program Files (x86)\\Intel\\oneAPI\\tbb\\latest\\bin\\tcm.dll",
        "C:\\Program Files (x86)\\Intel\\oneAPI\\tbb\\latest\\bin\\libhwloc-15.dll",
    ];

    let target_dir = crate::get_cargo_target_dir().unwrap();

    for lib in libs {
        let src = Path::new(lib);
        let target = target_dir.join(src.file_name().unwrap());
        std::fs::copy(src, &target)
            .with_context(|| format!("Failed to copy lib file {} to target", src.display()))?;
        let examples = target_dir.join("examples").join(src.file_name().unwrap());
        safe_hard_link(&target, &examples)
            .with_context(|| format!("Failed to copy lib file {} to examples", target.display()))?;
        let deps = target_dir.join("deps").join(src.file_name().unwrap());
        safe_hard_link(&target, &deps)
            .with_context(|| format!("Failed to copy lib file {} to deps", target.display()))?;

        // link oneAPI libs
        if !build_shared_libs {
            let stem = src.file_stem().unwrap();
            let stem_str = stem.to_str().unwrap();

            // Remove the "lib" prefix if present
            let lib_name = if stem_str.starts_with("lib") {
                stem_str.strip_prefix("lib").unwrap_or(stem_str)
            } else {
                stem_str
            };
            println!("cargo:rustc-link-lib=dylib={}", lib_name);
        }
    }

    Ok(())
}

fn copy_files_with_pattern(parttenre: &str, out_dir: &Path) -> Result<(), anyhow::Error> {
    let files = glob::glob(parttenre)?.filter_map(Result::ok);
    for file in files {
        let filename = file.file_name().unwrap();
        let filename = filename.to_str().unwrap();
        let dst = out_dir.join(filename);

        safe_hard_link(&file, &dst)?;
    }
    Ok(())
}

fn copy_llava_libs(out_dir: &Path, build_shared_libs: bool) -> Result<(), anyhow::Error> {
    // llava_static or llava_shared
    const FILE_STEM: &str = "*llava*";

    let lib_dir = out_dir.join("lib");
    let build_dir = out_dir.join("build").join("examples").join("llava");
    if cfg!(windows) {
        if build_shared_libs {
            let pattern = format!("{}/*/{}{}", build_dir.display(), FILE_STEM, ".dll");
            copy_files_with_pattern(&pattern, &lib_dir)?;
        }
        let pattern = format!("{}/*/{}{}", build_dir.display(), FILE_STEM, ".lib");
        copy_files_with_pattern(&pattern, &lib_dir)?;
    } else if cfg!(target_os = "macos") {
        if build_shared_libs {
            let pattern = format!("{}/*/{}{}", build_dir.display(), FILE_STEM, ".dylib");
            copy_files_with_pattern(&pattern, &lib_dir)?;
        } else {
            let pattern = format!("{}/*/{}{}", build_dir.display(), FILE_STEM, ".a");
            copy_files_with_pattern(&pattern, &lib_dir)?;
        }
    } else {
        if build_shared_libs {
            let pattern = format!("{}/*/{}{}", build_dir.display(), FILE_STEM, ".so");
            copy_files_with_pattern(&pattern, &lib_dir)?;
        } else {
            let pattern = format!("{}/*/{}{}", build_dir.display(), FILE_STEM, ".a");
            copy_files_with_pattern(&pattern, &lib_dir)?;
        }
    };
    Ok(())
}
