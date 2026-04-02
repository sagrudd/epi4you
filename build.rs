use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

const DOCS_IMAGE_TAG: &str = "epi4you-docs:almalinux9";
const DOCS_SKIP_ENV: &str = "EPI4YOU_SKIP_DOCS";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=docs/Makefile");
    println!("cargo:rerun-if-changed=docs/requirements.txt");
    println!("cargo:rerun-if-changed=docs/Dockerfile.docs");
    println!("cargo:rerun-if-changed=docs/source");

    if env::var_os(DOCS_SKIP_ENV).is_some() {
        println!(
            "cargo:warning=Skipping Sphinx documentation build because {DOCS_SKIP_ENV} is set"
        );
        return;
    }

    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set"));
    let docs_dir = manifest_dir.join("docs");
    let docs_source = docs_dir.join("source");
    let docs_build = docs_dir.join("build");

    if build_docs_with_sphinx(&docs_source, &docs_build) {
        return;
    }

    for engine in ["docker", "podman"] {
        if build_docs_with_container(engine, &manifest_dir, &docs_dir) {
            return;
        }
    }

    panic!(
        "Unable to build documentation. Install sphinx-build, or ensure docker/podman can build docs/Dockerfile.docs, or set {DOCS_SKIP_ENV}=1 to skip."
    );
}

fn build_docs_with_sphinx(docs_source: &Path, docs_build: &Path) -> bool {
    Command::new("sphinx-build")
        .args(["-M", "html"])
        .arg(docs_source)
        .arg(docs_build)
        .status()
        .is_ok_and(|status| status.success())
}

fn build_docs_with_container(engine: &str, manifest_dir: &Path, docs_dir: &Path) -> bool {
    if !command_works(engine, ["--version"]) {
        return false;
    }

    if !ensure_docs_image(engine, docs_dir) {
        return false;
    }

    let mut command = Command::new(engine);
    command
        .args(["run", "--rm"])
        .args(user_args())
        .args(["-v", &format!("{}:/workspace", manifest_dir.display())])
        .args(["-w", "/workspace"])
        .arg(DOCS_IMAGE_TAG)
        .args(["make", "-C", "docs", "html"]);

    command.status().is_ok_and(|status| status.success())
}

fn ensure_docs_image(engine: &str, docs_dir: &Path) -> bool {
    if command_works(engine, ["image", "inspect", DOCS_IMAGE_TAG]) {
        return true;
    }

    Command::new(engine)
        .args(["build", "-t", DOCS_IMAGE_TAG, "-f", "Dockerfile.docs", "."])
        .current_dir(docs_dir)
        .status()
        .is_ok_and(|status| status.success())
}

fn command_works<const N: usize>(program: &str, args: [&str; N]) -> bool {
    Command::new(program)
        .args(args)
        .status()
        .is_ok_and(|status| status.success())
}

fn user_args() -> Vec<String> {
    let uid = Command::new("id").arg("-u").output().ok();
    let gid = Command::new("id").arg("-g").output().ok();

    match (uid, gid) {
        (Some(uid), Some(gid)) if uid.status.success() && gid.status.success() => {
            let uid = String::from_utf8_lossy(&uid.stdout).trim().to_string();
            let gid = String::from_utf8_lossy(&gid.stdout).trim().to_string();
            if !uid.is_empty() && !gid.is_empty() {
                return vec!["--user".into(), format!("{uid}:{gid}")];
            }
            Vec::new()
        }
        _ => Vec::new(),
    }
}
