# Memory

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
