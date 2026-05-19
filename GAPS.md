# Gaps

The following features or tasks are identified for future work:

- **Shell script building**: The `scripts` subdirectory contains a utility shell script (`update-agentic-bootstrap`) which is not tracked under Bazel, as it functions as an external installer setup utility.
- **Bazel CI execution**: Setting up a GitHub Actions workflow or another CI pipeline to run `bazel test //...` on pull requests to ensure build integrity going forward.
- **Buildifier/Linter integration**: Integrating a Bazel code formatter (like `buildifier`) into local pre-commit hooks or just lint scripts when it becomes available.
- **Concurrent/Multithreaded Benchmarking**: The current Rust disk benchmark is single-threaded. Adding multi-threaded parallel read support would enable testing disk performance under high queue depth and concurrency conditions.


