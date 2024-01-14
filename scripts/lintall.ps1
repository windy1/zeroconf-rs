Set-PSDebug -Trace 1

$exit_code = 0

cargo clippy -- -D warnings
if ($LASTEXITCODE -ne 0) { $exit_code = -1; }
cargo clippy --all-targets --all-features -- -D warnings
if ($LASTEXITCODE -ne 0) { $exit_code = -1; }

cd examples
cargo clippy -- -D warnings
if ($LASTEXITCODE -ne 0) { $exit_code = -1; }
cargo clippy --all-targets --all-features -- -D warnings
if ($LASTEXITCODE -ne 0) { $exit_code = -1; }

Exit $exit_code
