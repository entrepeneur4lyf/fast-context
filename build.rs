extern crate napi_build;

fn main() {
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_NODEJS");

    if std::env::var_os("CARGO_FEATURE_NODEJS").is_some() {
        let _ = std::panic::catch_unwind(|| {
            napi_build::setup();
        });
    }
}
