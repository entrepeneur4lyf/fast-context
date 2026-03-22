extern crate napi_build;

fn main() {
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_NODEJS");

    if std::env::var_os("CARGO_FEATURE_NODEJS").is_some() {
        let result = std::panic::catch_unwind(|| {
            napi_build::setup();
        });

        if result.is_err() {
            println!("cargo:warning=Skipping napi-build Windows setup because libnode.dll was not found in the current environment");
        }
    }
}
