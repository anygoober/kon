fn main() {
    #[cfg(feature = "bindgen")]
    uniffi::uniffi_bindgen_main();

    #[cfg(not(feature = "bindgen"))]
    panic!("feature not enabled: bindgen\nuse --features bindgen");
}
