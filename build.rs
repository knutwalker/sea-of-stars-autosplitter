fn main() {
    println!("cargo::rerun-if-env-changed=SOS_AS_DUMMY");
    println!("cargo::rustc-check-cfg=cfg(dummy)");

    println!("cargo::rerun-if-env-changed=SOS_AS_VARS");
    println!("cargo::rustc-check-cfg=cfg(vars)");

    if let Ok("1") = std::env::var("SOS_AS_DUMMY").as_deref() {
        println!("cargo::rustc-cfg=dummy");
    }

    if let Ok("1") = std::env::var("SOS_AS_VARS").as_deref() {
        println!("cargo::rustc-cfg=vars");
    }
}
