use std::env;
use std::fs;
use std::path::PathBuf;

pub const DLL_FILE_NAME: &str = "bass.dll";

pub const BINDINGS_FILE_NAME: &str = "bindings.rs";

/// Get the filename for the prebuilt bindings for our target platform.
fn prebuilt_lib_binary_filename() -> &'static str {
    // Note: We can't use cfg(target_os) because this breaks when cross-compiling.
    let target_os = env::var("CARGO_CFG_TARGET_OS");

    match target_os.as_ref().map(|x| &**x) {
        Ok("linux") => "bass.lib",
        Ok("windows") => "libbass.so",
        Ok("macos") => "libbass.dylib",
        Ok(unsupported_os) => panic!(
            "Unsupported target os: \"{}\". Not sure what to link against, so we'll do nothing.",
            unsupported_os
        ),
        Err(err) => panic!("Error reading target os for build: {}", err),
    }
}

/// Get the filename for the prebuilt bindings for our target platform.
#[cfg(not(feature = "gen-bindings"))]
fn prebuilt_bindings_filename() -> &'static str {
    // Note: We can't use cfg(target_os) because this breaks when cross-compiling.
    let target_os = env::var("CARGO_CFG_TARGET_OS");

    match target_os.as_ref().map(|x| &**x) {
        Ok("linux") => "bindings_lnx.rs",
        Ok("windows") => "bindings_win.rs",
        Ok(unsupported_os) => panic!(
            "Unsupported target os for prebuilt bindings: \"{}\". Use the bindgen feature instead",
            unsupported_os
        ),
        Err(err) => panic!("Error reading target os for build: {}", err),
    }
}

// If binding generation is enabled, run bindgen to generate fresh bindings
#[cfg(feature = "gen-bindings")]
fn process_bindings(_lib_path: &PathBuf, out_path: &PathBuf) {
    println!("Running bindgen!");

    println!("cargo:rerun-if-changed=lib/bass.h");
    let bindings = bindgen::Builder::default()
        .header("lib/bass.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .derive_default(true)
        .derive_debug(true)
        .whitelist_function("BASS.*")
        .whitelist_type("BASS.*")
        .whitelist_var("BASS.*")
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join(BINDINGS_FILE_NAME))
        .expect("Couldn't write bindings!");
}

// If binding generation is not enabled, copy the prebuilt bindings from the lib folder
#[cfg(not(feature = "gen-bindings"))]
fn process_bindings(lib_path: &PathBuf, out_path: &PathBuf) {
    println!(
        "Using prebuilts: {} -> {}!",
        prebuilt_bindings_filename(),
        BINDINGS_FILE_NAME
    );
    fs::copy(
        lib_path
            .join(prebuilt_bindings_filename())
            .to_str()
            .unwrap(),
        out_path.join(BINDINGS_FILE_NAME).to_str().unwrap(),
    )
    .expect("Failed to copy prebuilt bindings to output directory");
}

fn main() {
    let lib_path = PathBuf::from("lib");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Copy the library into the output folder and instruct cargo to link against it
    let lib_filename = prebuilt_lib_binary_filename();
    fs::copy(
        lib_path.join(lib_filename).to_str().unwrap(),
        out_path.join(lib_filename).to_str().unwrap(),
    )
    .unwrap_or_else(|err| panic!(
        "Failed to copy native lib (\"{}\") to output directory: {}",
        lib_filename,
        err
    ));

    println!("cargo:rustc-link-lib=bass");
    println!(
        "cargo:rustc-link-search=native={}",
        out_path.to_str().unwrap()
    );

    // On Windows, we also need to copy a DLL to the output folder
    if env::var("CARGO_CFG_TARGET_OS") == Ok("windows".to_string()) {
        fs::copy(
            lib_path.join(DLL_FILE_NAME).to_str().unwrap(),
            out_path.join(DLL_FILE_NAME).to_str().unwrap(),
        )
        .unwrap_or_else(|err| panic!(
            "Failed to copy native DLL (\"{}\") to output directory: {}",
            DLL_FILE_NAME,
            err
        ));
    }

    // Generate the Rust bindings
    process_bindings(&lib_path, &out_path);
}
