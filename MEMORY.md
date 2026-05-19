# Memory

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

