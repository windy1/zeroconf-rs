Set-PSDebug -Trace 1

cargo build --workspace --verbose
if ($LASTEXITCODE -ne 0) { Exit $LASTEXITCODE }

cd examples
cargo build --workspace --verbose
if ($LASTEXITCODE -ne 0) { Exit $LASTEXITCODE }
