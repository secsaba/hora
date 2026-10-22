fn main() {
    cc::Build::new()
        .files(
            [
                "swecl", "swedate", "swehel", "swehouse", "swejpl", "swemmoon", "swemplan",
                "sweph", "swephlib",
            ]
            .map(|f| format!("swisseph/{f}.c")),
        )
        // Drops swe_get_library_path, which needs dladdr (ADR 0001 keeps it out anyway). TLSOFF stays undefined.
        .define("NO_SWE_GLP", None)
        // Vendored upstream code: its warnings are not ours to fix.
        .warnings(false)
        .compile("swisseph");
    println!("cargo::rerun-if-changed=swisseph");
}
