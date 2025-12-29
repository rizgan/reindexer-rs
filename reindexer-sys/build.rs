use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/ffi.cpp");
    println!("cargo:rerun-if-changed=src/ffi.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../vendor/reindexer-5.9.0");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    let vendor_dir = manifest_dir.parent().unwrap().join("vendor");
    let reindexer_src = vendor_dir.join("reindexer-5.9.0");
    let leveldb_src = vendor_dir.join("leveldb-1.23");
    let snappy_src = vendor_dir.join("snappy-1.1.10");

    // Build directories
    let snappy_build = out_dir.join("snappy-build");
    let leveldb_build = out_dir.join("leveldb-build");
    let reindexer_build = out_dir.join("reindexer-build");

    std::fs::create_dir_all(&snappy_build).unwrap();
    std::fs::create_dir_all(&leveldb_build).unwrap();
    std::fs::create_dir_all(&reindexer_build).unwrap();

    // Build Snappy (static)
    if !snappy_build.join("snappy.lib").exists() && !snappy_build.join("libsnappy.a").exists() {
        println!("cargo:warning=Building Snappy...");
        let status = Command::new("cmake")
            .current_dir(&snappy_build)
            .args(&[
                "-DCMAKE_BUILD_TYPE=Release",
                "-DSNAPPY_BUILD_TESTS=OFF",
                "-DSNAPPY_BUILD_BENCHMARKS=OFF",
                "-DBUILD_SHARED_LIBS=OFF",
                "-DCMAKE_POSITION_INDEPENDENT_CODE=ON",
                snappy_src.to_str().unwrap(),
            ])
            .status()
            .expect("Failed to configure Snappy");
        assert!(status.success(), "Snappy CMake configuration failed");

        let status = Command::new("cmake")
            .current_dir(&snappy_build)
            .args(&["--build", ".", "--config", "Release"])
            .status()
            .expect("Failed to build Snappy");
        assert!(status.success(), "Snappy build failed");

        // CMake generates snappy-stubs-public.h into the build tree. Reindexer expects
        // it to be discoverable via SNAPPY_INCLUDE_DIR, so copy it next to snappy.h
        // to avoid missing-header errors on Windows generators.
        let generated_header = snappy_build.join("snappy-stubs-public.h");
        let dest_header = snappy_src.join("snappy-stubs-public.h");
        if generated_header.exists() {
            let _ = std::fs::copy(&generated_header, &dest_header);
        }
    }

    // Build LevelDB (static)
    if !leveldb_build.join("libleveldb.lib").exists() && !leveldb_build.join("libleveldb.a").exists() {
        println!("cargo:warning=Building LevelDB...");
        let status = Command::new("cmake")
            .current_dir(&leveldb_build)
            .args(&[
                "-DCMAKE_BUILD_TYPE=Release",
                "-DLEVELDB_BUILD_TESTS=OFF",
                "-DLEVELDB_BUILD_BENCHMARKS=OFF",
                "-DBUILD_SHARED_LIBS=OFF",
                "-DCMAKE_POSITION_INDEPENDENT_CODE=ON",
                &format!("-DCMAKE_PREFIX_PATH={}", snappy_build.display()),
                leveldb_src.to_str().unwrap(),
            ])
            .status()
            .expect("Failed to configure LevelDB");
        assert!(status.success(), "LevelDB CMake configuration failed");

        let status = Command::new("cmake")
            .current_dir(&leveldb_build)
            .args(&["--build", ".", "--config", "Release"])
            .status()
            .expect("Failed to build LevelDB");
        assert!(status.success(), "LevelDB build failed");
    }

    // Build Reindexer (static, builtin only)
    if !reindexer_build.join("cpp_src").join("Release").join("reindexer.lib").exists() && 
       !reindexer_build.join("cpp_src").join("libreindexer.a").exists() {
        println!("cargo:warning=Building Reindexer with builtin vector indexes...");
        
        // Установить пути к уже собранным библиотекам
        let snappy_lib = if cfg!(target_os = "windows") {
            snappy_build.join("Release").join("snappy.lib")
        } else {
            snappy_build.join("libsnappy.a")
        };
        
        let leveldb_lib = if cfg!(target_os = "windows") {
            leveldb_build.join("Release").join("leveldb.lib")
        } else {
            leveldb_build.join("libleveldb.a")
        };
        
        let snappy_inc = snappy_src.clone();
        let leveldb_inc = leveldb_src.join("include");
        let cmake_modules = vendor_dir.clone();
        
        // Конвертируем пути в Unix-стиль для CMake (прямые слэши)
        let cmake_modules_str = cmake_modules.to_str().unwrap().replace("\\", "/");
        let leveldb_lib_str = leveldb_lib.to_str().unwrap().replace("\\", "/");
        let leveldb_inc_str = leveldb_inc.to_str().unwrap().replace("\\", "/");
        let snappy_lib_str = snappy_lib.to_str().unwrap().replace("\\", "/");
        let snappy_inc_str = snappy_inc.to_str().unwrap().replace("\\", "/");
        
        let status = Command::new("cmake")
            .current_dir(&reindexer_build)
            .env("SNAPPY_INCLUDE_DIR", &snappy_inc_str)
            .env("SNAPPY_LIBRARY", &snappy_lib_str)
            .env("LEVELDB_INCLUDE_DIR", &leveldb_inc_str)
            .env("LEVELDB_LIBRARY", &leveldb_lib_str)
            .args(&[
                &format!("-DCMAKE_MODULE_PATH={}", cmake_modules_str),
                "-DCMAKE_BUILD_TYPE=Release",
                "-DBUILD_SHARED_LIBS=OFF",
                "-DCMAKE_POSITION_INDEPENDENT_CODE=ON",
                "-DENABLE_GRPC=OFF",
                "-DENABLE_ROCKSDB=OFF",
                "-DWITH_PYTHON=OFF",
                "-DWITH_CPPTRACE=OFF",
                "-DBUILD_ANN_INDEXES=builtin",  
                "-DENABLE_OPENSSL=OFF",
                "-DENABLE_TCMALLOC=OFF",
                "-DENABLE_JEMALLOC=OFF",
                &format!("-DLevelDB_LIBRARY={}", leveldb_lib_str),
                &format!("-DLevelDB_INCLUDE_DIR={}", leveldb_inc_str),
                &format!("-DSNAPPY_LIBRARY={}", snappy_lib_str),
                &format!("-DSNAPPY_INCLUDE_DIR={}", snappy_inc_str),
                reindexer_src.to_str().unwrap(),
            ])
            .status()
            .expect("Failed to configure Reindexer");
        assert!(status.success(), "Reindexer CMake configuration failed");

        let status = Command::new("cmake")
            .current_dir(&reindexer_build)
            .args(&["--build", ".", "--config", "Release", "--target", "reindexer", "--parallel", "2"])
            .status()
            .expect("Failed to build Reindexer");
        assert!(status.success(), "Reindexer build failed");
    }

    // Build our FFI wrapper
    let reindexer_cpp_src = reindexer_src.join("cpp_src");
    let reindexer_vendor = reindexer_cpp_src.join("vendor");

    cc::Build::new()
        .cpp(true)
        .file("src/ffi.cpp")
        .flag_if_supported("-std=c++20")
        .flag_if_supported("/std:c++20")
        .include(&reindexer_vendor)
        .include(&reindexer_cpp_src)
        .include("src")
        .warnings(false)
        .compile("reindexer_ffi");

    // Link libraries
    let reindexer_lib_path = if cfg!(target_os = "windows") {
        reindexer_build.join("cpp_src").join("Release")
    } else {
        reindexer_build.join("cpp_src")
    };
    
    println!("cargo:rustc-link-search=native={}", reindexer_lib_path.display());
    println!("cargo:rustc-link-search=native={}", leveldb_build.join("Release").display());
    println!("cargo:rustc-link-search=native={}", snappy_build.join("Release").display());
    
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=static=reindexer");
        println!("cargo:rustc-link-lib=static=friso_dict_resources");
        println!("cargo:rustc-link-lib=static=leveldb");
        println!("cargo:rustc-link-lib=static=snappy");
    } else {
        println!("cargo:rustc-link-lib=static=reindexer");
        println!("cargo:rustc-link-lib=static=friso_dict_resources");
        println!("cargo:rustc-link-lib=static=leveldb");
        println!("cargo:rustc-link-lib=static=snappy");
    }

    // System libraries
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=dylib=ws2_32");
        println!("cargo:rustc-link-lib=dylib=shlwapi");
    } else if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=dylib=pthread");
        println!("cargo:rustc-link-lib=dylib=dl");
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }
}
