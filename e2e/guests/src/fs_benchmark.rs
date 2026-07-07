use std::fs;
use std::io::{Read, Write};
use std::path::Path;

/// Number of files to create for multi-file benchmarks.
const FILE_COUNT: usize = 100;
/// Size of data written per file (bytes).
const FILE_SIZE: usize = 4096;
/// Name of the benchmark working directory (created under cwd).
const BENCH_DIR: &str = "fs_bench_workdir";

fn main() {
    let bench_dir = Path::new(BENCH_DIR);

    // Clean up from any prior run.
    if bench_dir.exists() {
        fs::remove_dir_all(bench_dir).expect("cleanup: remove_dir_all");
    }

    // -- 1. Create directory tree -------------------------------------------
    create_dirs(bench_dir);

    // -- 2. Sequential file creation + write --------------------------------
    let data = generate_data(FILE_SIZE);
    sequential_write(bench_dir, &data);

    // -- 3. Sequential file read --------------------------------------------
    sequential_read(bench_dir);

    // -- 4. Random-order file read ------------------------------------------
    random_read(bench_dir);

    // -- 5. File metadata / stat --------------------------------------------
    stat(bench_dir);

    // -- 6. Directory listing -----------------------------------------------
    readdir(bench_dir);

    // -- 7. Rename files ----------------------------------------------------
    rename_files(bench_dir);

    // -- 8. Delete files (cleanup) ------------------------------------------
    delete_files(bench_dir);

    // -- 9. Remove directories ----------------------------------------------
    remove_dirs(bench_dir);
}

// ---------------------------------------------------------------------------
// Individual operations
// ---------------------------------------------------------------------------

/// Create a set of nested directories.
fn create_dirs(base: &Path) {
    for i in 0..10 {
        fs::create_dir_all(base.join(format!("sub_{i}"))).expect("create_dir_all");
    }
}

/// Write `FILE_COUNT` files of `FILE_SIZE` bytes each.
fn sequential_write(base: &Path, data: &[u8]) {
    for i in 0..FILE_COUNT {
        let path = file_path(base, i);
        let mut f = fs::File::create(&path).expect("create file");
        f.write_all(data).expect("write_all");
    }
}

/// Read every file back sequentially in creation order.
fn sequential_read(base: &Path) {
    let mut buf = vec![0u8; FILE_SIZE];
    for i in 0..FILE_COUNT {
        let path = file_path(base, i);
        let mut f = fs::File::open(&path).expect("open file");
        f.read_exact(&mut buf).expect("read_exact");
    }
}

/// Read files in a pseudo-random order (simple LCG).
fn random_read(base: &Path) {
    let order = lcg_permutation(FILE_COUNT);
    let mut buf = vec![0u8; FILE_SIZE];
    for &i in &order {
        let path = file_path(base, i);
        let mut f = fs::File::open(&path).expect("open file");
        f.read_exact(&mut buf).expect("read_exact");
    }
}

/// Stat (metadata) every file.
fn stat(base: &Path) {
    for i in 0..FILE_COUNT {
        let path = file_path(base, i);
        let meta = fs::metadata(&path).expect("metadata");
        // Use the value so the compiler does not optimise the call away.
        assert_eq!(meta.len(), FILE_SIZE as u64);
    }
}

/// Read the directory listing of the base dir (files are spread across subdirs).
fn readdir(base: &Path) {
    for entry in fs::read_dir(base).expect("read_dir") {
        let entry = entry.expect("dir entry");
        // Touch the name to prevent optimisation.
        let _ = entry.file_name();
    }
}

/// Rename every file (append `.renamed`).
fn rename_files(base: &Path) {
    for i in 0..FILE_COUNT {
        let src = file_path(base, i);
        let dst = src.with_extension("renamed");
        fs::rename(&src, &dst).expect("rename");
    }
}

/// Delete every renamed file.
fn delete_files(base: &Path) {
    for i in 0..FILE_COUNT {
        let path = file_path(base, i).with_extension("renamed");
        fs::remove_file(&path).expect("remove_file");
    }
}

/// Remove the directory tree created by `create_dirs`.
fn remove_dirs(base: &Path) {
    fs::remove_dir_all(base).expect("remove_dir_all");
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Generate deterministic data of the given length.
fn generate_data(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i % 251) as u8).collect()
}

/// Map a file index to a path, distributing files across subdirectories.
fn file_path(base: &Path, index: usize) -> std::path::PathBuf {
    let subdir = index % 10;
    base.join(format!("sub_{subdir}"))
        .join(format!("file_{index:04}.dat"))
}

/// Produce a pseudo-random permutation of `0..n` using a simple LCG.
fn lcg_permutation(n: usize) -> Vec<usize> {
    // Fisher-Yates shuffle driven by a basic LCG (no external rand crate needed).
    let mut v: Vec<usize> = (0..n).collect();
    let mut state: u64 = 12345; // fixed seed for reproducibility
    for i in (1..n).rev() {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let j = (state >> 33) as usize % (i + 1);
        v.swap(i, j);
    }
    v
}
