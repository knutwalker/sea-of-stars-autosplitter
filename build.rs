fn main() {
    println!("cargo::rerun-if-env-changed=SOS_AS_DEBUG");
    println!("cargo::rustc-check-cfg=cfg(debugger)");

    println!("cargo::rerun-if-env-changed=SOS_AS_DUMMY");
    println!("cargo::rustc-check-cfg=cfg(dummy)");

    if let Ok("1") = std::env::var("SOS_AS_DEBUG").as_deref() {
        println!("cargo::rustc-cfg=debugger");
    }
    if let Ok("1") = std::env::var("SOS_AS_DUMMY").as_deref() {
        println!("cargo::rustc-cfg=dummy");
    }
}
