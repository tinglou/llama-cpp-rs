//! push compiler flags
//! * if feature is set manually, then force to push the flags
//! * detect 

use cc::Build;

/// true if at least one force feature is turned on
const FORCE_ARCH: bool = cfg!(any(
    force_avx1,
    force_avx2,
    force_avx512f,
    force_avx512bw,
    force_avx512vbmi,
    force_avx512vnni
));

macro_rules! x86_feature {
    ($feature:expr) => {
        {
            // 生成条件表达式
            // (!FORCE_ARCH && is_x86_feature_detected!(stringify!($feature))) 
            //|| cfg!(feature = concat!("force_", $feature))

        }
    };
}
fn test() {
let avx = x86_feature!("avx1");
}

/// Add platform appropriate flags and definitions based on enabled features.
fn push_feature_flags(cx: &mut Build, cxx: &mut Build) {
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
            if cfg!(target_feature = "avx512f") {
                cx.flag("-mavx512f");
                cxx.flag("-mavx512f");

                if cfg!(target_feature = "avx512bw") {
                    cx.flag("-mavx512bw");
                    cxx.flag("-mavx512bw");
                }

                if cfg!(target_feature = "avx512vbmi") {
                    cx.flag("-mavx512vbmi");
                    cxx.flag("-mavx512vbmi");
                }

                if cfg!(target_feature = "avx512vnni") {
                    cx.flag("-mavx512vnni");
                    cxx.flag("-mavx512vnni");
                }
            }

            if cfg!(target_feature = "avx2") {
                cx.flag("-mavx2");
                cxx.flag("-mavx2");
            }

            if cfg!(target_feature = "avx") {
                cx.flag("-mavx");
                cxx.flag("-mavx");
            }
        } else if cfg!(target_family = "windows") {
            if (!FORCE_ARCH && is_x86_feature_detected!("avx512f")) || cfg!(feature = "force_avx512f") {

                cx.flag("/arch:AVX512");
                cxx.flag("/arch:AVX512");

                if cfg!(target_feature = "avx512vbmi") {
                    cx.define("__AVX512VBMI__", None);
                    cxx.define("__AVX512VBMI__", None);
                }

                if cfg!(target_feature = "avx512vnni") {
                    cx.define("__AVX512VNNI__", None);
                    cxx.define("__AVX512VNNI__", None);
                }
            } else if cfg!(target_feature = "avx2") {
                cx.flag("/arch:AVX2");
                cxx.flag("/arch:AVX2");
            } else if cfg!(target_feature = "avx") {
                cx.flag("/arch:AVX");
                cxx.flag("/arch:AVX");
            }
        }
    }
}
