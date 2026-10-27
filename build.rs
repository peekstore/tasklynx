fn main() {
    println!("cargo:rerun-if-changed=assets/tasklynx.ico");
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/tasklynx.ico");
        res.set("ProductName", "Tasklynx");
        res.set("FileDescription", "Tasklynx developer task manager");
        res.set("CompanyName", "RK Riad Khan");
        res.compile().expect("failed to embed Windows icon");
    }
}
