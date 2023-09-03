fn main() {
    println!("cargo:rerun-if-env-changed=SOS_AS_DEBUG");
    if let Ok("1") = std::env::var("SOS_AS_DEBUG").as_deref() {
        println!("cargo:rustc-cfg=debugger");
    }
}
