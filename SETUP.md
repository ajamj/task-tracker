# Setup Instructions for tt CLI

## Prerequisites

- Rust toolchain (rustc, cargo) - https://rustup.rs/
- Git - Already installed at `C:\Program Files\Git\bin\git.exe`

## Quick Setup

### 1. Initialize Git Repository

Open **Command Prompt** or **PowerShell** and run:

```cmd
cd D:\GRC-Ajam\rust-playground

"C:\Program Files\Git\bin\git.exe" init
"C:\Program Files\Git\bin\git.exe" add .
"C:\Program Files\Git\bin\git.exe" commit -m "feat: complete MVP implementation (all 8 tasks)"
```

Or if git is in your PATH:

```cmd
cd D:\GRC-Ajam\rust-playground
git init
git add .
git commit -m "feat: complete MVP implementation (all 8 tasks)"
```

### 2. Build the Project

```cmd
cargo build
```

### 3. Run Tests

```cmd
cargo test
```

### 4. Lint (Optional)

```cmd
cargo clippy -- -D warnings
cargo fmt --check
```

### 5. Test the CLI

```cmd
# Create a test workspace
mkdir test-worklog
cd test-worklog

# Initialize (using the built binary)
..\.target\debug\tt.exe init

# Or install globally
cargo install --path .

# Then use tt directly
tt init
tt add "Refactor config loader" --due 2026-04-03 --tag rust --tag cli
tt ls
tt start tt-000001
tt log "Worked on tt-000001: initial implementation"
tt done tt-000001
tt report week
```

## Troubleshooting

### Git Not Found

If you get "git is not recognized", use the full path:

```cmd
"C:\Program Files\Git\bin\git.exe" --version
```

### Cargo Not Found

Make sure Rust is installed and in your PATH:

```cmd
rustup --version
cargo --version
```

### Build Errors

If you see compilation errors, check your Rust version:

```cmd
rustc --version
```

Minimum required: 1.75 (specified in Cargo.toml)

## Verify Installation

After building, verify the binary exists:

```cmd
dir target\debug\tt.exe
```

Or after install:

```cmd
tt --version
```

## Next Steps

1. Run the setup commands above
2. Test all CLI commands
3. If everything works, you're ready to use `tt` daily!
4. Consider pushing to GitHub for backup/sharing

---

**Note:** The shell commands in the AI agent environment don't work properly in this setup. Please run all commands manually in your terminal.
