use std::{env, path::Path};

use anyhow::Context;
use cmake::Config;

pub fn pre_cmake_build(config: &mut Config) -> anyhow::Result<()> {
    let target = env::var("TARGET")?;
    // println!("cargo:warning=[DEBUG] {}", target);
    config.define("LLAMA_BUILD_COMMON", "ON");
    config.define("LLAMA_BUILD_EXAMPLES", "ON");
    if cfg!(target_os = "macos") && target.contains("x86_64") {
        // macOS x86_64 doesn't support OpenMP and Metal
        config.define("GGML_OPENMP", "OFF");
        config.define("GGML_METAL", "OFF");
        config.define("GGML_METAL_EMBED_LIBRARY", "OFF");
    }

    Ok(())
}

pub fn post_cmake_build(out_dir: &Path, build_shared_libs: bool) -> anyhow::Result<()> {
    const FILE_STEM_SHARED: &str = "llava_shared";
    const FILE_STEM_STATIC: &str = "llava_static";

    let lib_dir = out_dir.join("lib");
    let build_dir = out_dir.join("build/examples/llava/");
    if cfg!(windows) {
        if build_shared_libs {
            let src = build_dir.join(format!("Release/{}{}", FILE_STEM_SHARED, ".dll"));
            let dst = lib_dir.join(format!("{}{}", FILE_STEM_SHARED, ".dll"));
            std::fs::copy(&src, &dst)
                .with_context(|| format!("Failed to copy lib file {}", src.display()))?;
        }
        let src = build_dir.join(format!("Release/{}{}", FILE_STEM_STATIC, ".lib"));
        let dst = lib_dir.join(format!("{}{}", FILE_STEM_STATIC, ".lib"));
        std::fs::copy(&src, &dst)
            .with_context(|| format!("Failed to copy lib file {}", src.display()))?;
    } else if cfg!(target_os = "macos") {
        if build_shared_libs {
            let src = build_dir.join(format!("lib{}{}", FILE_STEM_SHARED, ".dylib"));
            let dst = lib_dir.join(format!("lib{}{}", FILE_STEM_SHARED, ".dylib"));
            std::fs::copy(&src, &dst)
                .with_context(|| format!("Failed to copy lib file {}", src.display()))?;
        } else {
            let src = build_dir.join(format!("lib{}{}", FILE_STEM_STATIC, ".a"));
            let dst = lib_dir.join(format!("lib{}{}", FILE_STEM_STATIC, ".a"));
            std::fs::copy(&src, &dst)
                .with_context(|| format!("Failed to copy lib file {}", src.display()))?;
        }
    } else {
        if build_shared_libs {
            let src = build_dir.join(format!("lib{}{}", FILE_STEM_SHARED, ".so"));
            let dst = lib_dir.join(format!("lib{}{}", FILE_STEM_SHARED, ".so"));
            std::fs::copy(&src, &dst)
                .with_context(|| format!("Failed to copy lib file {}", src.display()))?;
        } else {
            let src = build_dir.join(format!("lib{}{}", FILE_STEM_STATIC, ".a"));
            let dst = lib_dir.join(format!("lib{}{}", FILE_STEM_STATIC, ".a"));
            std::fs::copy(&src, &dst)
                .with_context(|| format!("Failed to copy lib file {}", src.display()))?;
        }
    };
    Ok(())
}
