fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = root
            .parent()
            .expect("invalid root directory")
            .parent()
            .expect("invalid workspace root directory");
        let icon_path = workspace_root.join("resources").join("images").join("icon.ico");
        res.set_icon(icon_path.to_str().expect("failed to convert path to str"));
        if let Err(e) = res.compile() {
            println!("cargo:warning=Failed to compile resource file: {e}");
        };
    }
}
