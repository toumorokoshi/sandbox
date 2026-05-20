# Memory

## 2026-05-20: Refactored Parquet vs Lance Benchmark Read Logic

### Context
Refactored the Parquet and Lance read implementations from inline blocks inside the main loop into separate, modular functions (`read_parquet` and `read_lance`).

### Major Decisions and Changes
1. **Created ReadMetrics Struct**: Introduced a shared metrics container `ReadMetrics` to pass read duration, throughput (MB/s), and count of rows read back to the main loop.
2. **Refactored Read Functions**: Moved the format-specific reading and stream traversal logic into helper functions `read_parquet` and `read_lance`.
3. **Suppressed Dead Code Warnings**: Applied `#[allow(dead_code)]` to the struct to prevent unused field warnings for fields intended for downstream metrics tracking.

## 2026-05-20: Resolved Bazel Build Failures for Parquet vs Lance Benchmark

### Context
Fixed compilation failures when building `//parquet_vs_lance_benchmark:parquet_vs_lance_benchmark` through Bazel. These failures were caused by: (1) `let_chains` compilation error in the `comfy-table` dependency under the older toolchain, (2) version mismatches between the direct `arrow` dependency and `lance`'s transitive `arrow` version, and (3) missing `protoc` compiler inside Bazel sandbox for the `lance-index` build script.

### Major Decisions and Changes
1. **Toolchain Upgrade**: Updated the Rust toolchain versions inside `MODULE.bazel` from `1.85.0` to `1.93.1`. This supports stable `let_chains` which are used inside `comfy-table v7.2.2`.
2. **Arrow & Parquet Version Alignment**: Downgraded direct `arrow` and `parquet` dependencies inside `parquet_vs_lance_benchmark/Cargo.toml` from `54.2.0` to `53.4.1` to align with the exact versions expected by `lance 0.22.0`, resolving a `RecordBatchReader` trait mismatch.
3. **Bazel Environment Configuration**:
   - Created `.bazelrc` to pass `PATH` (including `/opt/homebrew/bin`) and the `PROTOC` variable into the Bazel action environment, allowing the `lance-index` build script to locate `protoc` successfully on macOS.
   - Created `.bazelignore` to ignore `.venv` and `target` to optimize Bazel query traversals.
4. **Code Warnings Cleanups**: Resolved Rust compiler/Clippy warnings in the benchmark source code.

## 2026-05-19: Added Parquet vs Lance Format Benchmark and Cargo Workspace

### Context
Created a benchmark evaluating read throughput of Parquet vs Lance over varying column payload sizes (1 KB to 10 MB). Integrated the new crate under the unified Bazel build.

### Major Decisions and Changes
1. **Format Benchmark Crate**: Initialized `parquet_vs_lance_benchmark` utilizing `parquet`, `lance`, and `arrow` to measure variable-size column reads and outputs results in MB/s.
2. **Cargo Workspace Root**: Defined a root `Cargo.toml` workspace unifying both `io_benchmark` and `parquet_vs_lance_benchmark`.
3. **Bzlmod crate_universe Integration**: Refactored `MODULE.bazel` to generate the dependency graph from the single root `Cargo.lock` and member manifests, streamlining dependency sharing across Rust binaries.
4. **Bazel Targetization**: Created `parquet_vs_lance_benchmark/BUILD.bazel` `rust_binary` to build the benchmark consistently within the hermetic sandbox.

## 2026-05-18: Enhanced Benchmark Loop Symmetries & Published Experiments

### Context
Made the benchmarks fully symmetric by enabling both sequential and random reads to loop sequentially/randomly back to offset 0 and cycle until target operations or durations are satisfied, and published premium experiments to a README directory dashboard.

### Major Decisions and Changes
1. **Dynamic Sequential Read Cycling**: Added logic to `run_sequential_benchmark` that seeks back to the start of the file (`seek(0)`) whenever EOF is reached, allowing it to loop continuously under exact operation targets (`--ops`) or duration bounds (`--duration`).
2. **Dynamic Balanced Mode**: Wrapped `duration` CLI argument inside `Option<Duration>`, dynamically defaulting both read benchmarks to execute exactly `num_blocks` operations (matching file size / block size) if no options are specified. This achieves perfectly comparable bytes read volumes and execution times under default conditions.
3. **Natively Derived Bytesize Parsing**: Removed custom suffix-normalizing helper functions in favor of `bytesize::ByteSize`'s native direct string parsing, updating the CLI documentation and binary formats to standard `128MiB` / `4KiB` styles.
4. **Comprehensive Readme Published**: Generated a detailed, premium `io_benchmark/README.md` containing absolute command-line usage details, CLI parameter references, and exhaustive performance analyses of our three SSD experiments (Cached vs Direct I/O, Small vs Large blocks).
5. **Quality Verification**: Verified all unit tests and hermetic execution sandboxes pass successfully across both Cargo and Bazel test runners.

## 2026-05-18: Migrated CLI Parsing to Clap

### Context
Replaced the manually hardcoded option parsing loop inside `io_benchmark` with the robust and modern `clap` struct-derive API.

### Major Decisions and Changes
1. **Added Clap Dependency**: Added `clap` with the `derive` feature in `Cargo.toml` and wired it through Bzlmod and `@crates//:clap` in `io_benchmark/BUILD.bazel`.
2. **Derived CLI Struct**: Refactored `Config` using `#[derive(Parser)]` and `#[arg(...)]` decorators. This gives professional, self-documenting CLI helper menus and usage validation for free.
3. **Custom Argument Value Parsers**: Developed custom validators (`parse_size_arg`, `parse_block_size_arg`, `parse_duration_arg`) integrating directly with `clap`'s type system to robustly translate formatted strings (e.g. `256M`, `4K`) into native Rust values.

## 2026-05-18: Integrated Rust Benchmark under Bazel Build

### Context
Converted the `io_benchmark` Rust project build to **Bazel 9.1.0** to ensure the entire repository (C++, Go, Python, and Rust) builds and runs under a unified hermetic system.

### Major Decisions and Changes
1. **rules_rust Integration**: Added `rules_rust` (version `0.70.0` for full Bazel 9 compatibility) to `MODULE.bazel`.
2. **Hermetic Rust Toolchain**: Configured `rust.toolchain` using standard version `1.85.0` to correctly support Rust's 2024 edition compilation.
3. **Crate Universe Cargo Lock Ingestion**: Configured `crate.from_cargo` to read direct/transitive dependencies (`libc`, `rand`, `anyhow`) directly from `io_benchmark/Cargo.toml` and `io_benchmark/Cargo.lock`, mapping them hermetically to `@crates//:`.
4. **Targetized Build files**: Created `io_benchmark/BUILD.bazel` with:
   - `rust_binary` for `io_benchmark` tool.
   - `rust_test` for `io_benchmark_test` which runs all inline unit tests hermetically inside Bazel's sandbox.
5. **Sandbox Compliance Verified**: Executed `bazel test` and `bazel run` to guarantee that C++ / Go / Python / Rust test targets pass cleanly across the entire workspace.

## 2026-05-18: Implemented Rust Disk I/O Benchmark Suite

### Context
Created a brand-new, robust, single-threaded disk I/O benchmarking tool rewritten in **Rust** to compare random reads versus sequential reads, detailing total throughput, IOPS, and average I/O latency.

### Major Decisions and Changes
1. **Rust Cargo Package Creation**: Initialized a standard binary Cargo package under the newly created `io_benchmark/` directory.
2. **Direct/Uncached I/O Support**: Integrated OS-specific direct file operations to bypass the OS Page Cache:
   - On **macOS**, uses `libc::fcntl` with the `F_NOCACHE` flag.
   - On **Linux**, uses `libc::fcntl` with the `O_DIRECT` flag.
   - Graceful runtime fallback and warning handling on unsupported operating systems.
3. **Robust Metrics Calculation**:
   - **Throughput**: Computes total bytes read divided by time elapsed (formatted dynamically as B/s, KiB/s, MiB/s, or GiB/s).
   - **IOPS**: Tracks total completed read operations divided by duration in seconds.
   - **Latency**: Measures elapsed time per read operation (formatted in s, ms, or µs).
4. **Flexible CLI Configurations**: Implemented size and duration parsing allowing values like `--size 128M`, `--block-size 4K`, `--duration 10`, `--ops 50000`, `--nocache`, and `--keep`.
5. **Comprehensive Unit Testing**: Added tests for all helper parsing and formatting functions inside `src/main.rs`.
6. **Linting and Formatting**: Checked with `cargo clippy` and formatted cleanly via `cargo fmt` with no compiler warnings.

## 2026-05-18: Repository Converted to Bazel


### Context
Converted the repository from language-specific ad-hoc setups to a unified, hermetic building system using **Bazel 9**.

### Major Decisions and Changes
1. **Bzlmod Integration**: Defined a modern `MODULE.bazel` configuration specifying direct dependencies for:
   - `eigen` (`3.4.0.bcr.3`)
   - `rules_go` (`0.60.0`)
   - `rules_python` (`1.7.0`)
   - `rules_cc` (`0.2.17`)
2. **Hermetic SDK Setup**: Configured a Go SDK downloader extension inside Bzlmod to ensure builds are self-contained and reproducible.
3. **C++ (Eigen) Subdirectory Targetization**:
   - Replaced custom `<eigen3/Eigen/Dense>` include style with standard header-only library style `<Eigen/Dense>`.
   - Created three standard `cc_binary` targets for the examples (`main-affine-to-matrix`, `main-matrix`, and `main-vector`).
   - Fixed an out-of-bounds array index crash in `eigen/main-vector.cpp` (changed `m(1)` to `m(0)` for the size 1 vector) to pass safely during runs.
4. **Go Subdirectory Targetization**:
   - Created two separate `go_binary` targets in `go/threading/BUILD.bazel` to cleanly compile and run `main.go` and `with_channels.go`.
5. **Python Subdirectory Targetization**:
   - Created two `py_test` targets and three `py_binary` targets in `python/BUILD.bazel`.
   - Fixed pre-existing logical assertion errors in `python/unittest_example.py` (multiplication typo) and `python/context-manager-generator.py` (incorrect assert target value) so that `bazel test //...` now runs cleanly with 100% test success.

## 2026-05-18: Added PyTorch Dependency and PyTorch Typing Target

### Context
The user wanted to add the PyTorch library to `pytorch_typing/pytorch_type_example.py` and build/run it under Bazel.

### Major Decisions and Changes
1. **Pip Bzlmod Extension Configuration**: Added rules_python `pip.parse` and hermetic `python.toolchain` configurations to `MODULE.bazel` to manage third-party PyPI dependencies.
2. **Requirements Locking**:
   - Created an empty `BUILD.bazel` at the root and defined a `compile_pip_requirements` target to manage Python requirements.
   - Declared `torch` in `requirements.txt` and compiled it into `requirements_lock.txt` using `bazel run //:requirements.update` to automatically capture and verify all transitive dependencies (e.g. `sympy`, `networkx`, `jinja2`).
3. **Pytorch Typing Target Creation**:
   - Created `pytorch_typing/BUILD.bazel` with a `py_binary` target for `pytorch_type_example` that references `@pip_deps` to gain access to PyTorch.
   - Updated the typo in `pytorch_typing/pytorch_type_example.py` (changed `import pytorch` to standard `import torch`) and added a comprehensive typed tensor transformation demo.

