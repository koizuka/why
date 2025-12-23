# why

Identify which package manager installed a command.

Similar to `npm why`, but for system-wide commands. When you run `why <command>`, it tells you how that command was installed (Homebrew, npm, apt, etc.).

## Installation

### From source

```bash
git clone https://github.com/koizuka/why.git
cd why
cargo install --path .
```

This installs the `why` binary to `~/.cargo/bin/`, which should already be in your PATH if you have Rust installed.

### From releases

Download the binary for your platform from [Releases](https://github.com/koizuka/why/releases).

## Usage

```bash
# Basic usage
why git

# Output:
# git was installed by: Homebrew (verified)
#   Package: git
#   Version: 2.51.2
#   Location: /opt/homebrew/Cellar/git/2.51.2/bin/git
```

### Options

```
-f, --format <FORMAT>  Output format [default: text]
                       - text: Human-readable output
                       - json: JSON output
                       - short: Just the package manager name
    --json             Output as JSON (shortcut for --format json)
-v, --verbose          Show detection steps
-h, --help             Print help
-V, --version          Print version
```

### Examples

```bash
# JSON output
why --json git
# {"manager_id":"homebrew","manager_name":"Homebrew","package_name":"git",...}

# Short output (useful for scripts)
why --format short git
# homebrew

# Verbose mode (shows detection process)
why -v git
# Resolving path for 'git'...
# Found at /opt/homebrew/bin/git
# Following symlink to /opt/homebrew/Cellar/git/2.51.2/bin/git
# Trying Homebrew...
# ✓ Matched: Homebrew
```

## Supported Package Managers

| Package Manager | Platform | Detection Method |
|-----------------|----------|------------------|
| Homebrew | macOS, Linux | Cellar path pattern |
| npm (global) | All | node_modules path |
| bun (global) | All | .bun/bin path |
| yarn (global) | All | .yarn/bin path |
| pnpm (global) | All | pnpm global path |
| Cargo | All | .cargo/bin path |
| pipx | All | pipx venvs path |
| go install | All | go/bin path |
| gem (RubyGems) | All | .gem/ruby path |
| mise | All | mise/installs path |
| Nix | macOS, Linux | /nix/store, .nix-profile path |
| apt | Linux (Debian/Ubuntu) | dpkg query |
| Snap | Linux | /snap/bin path |
| Chocolatey | Windows | ProgramData path |
| Winget | Windows | WindowsApps path |
| Scoop | Windows | scoop/apps path |
| System | All | OS standard paths |

## How It Works

1. **Resolve command path** - Uses `which` to find the command
2. **Follow symlinks** - Traces symlink chain to find the actual binary
3. **Pattern matching** - Matches paths against known package manager patterns
4. **Verification** - Optionally queries the package manager to confirm

## Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Check with clippy
cargo clippy
```

## Contributing

Contributions are welcome! Feel free to:

- Add support for new package managers
- Improve detection accuracy
- Fix bugs or add tests

## License

MIT License - see [LICENSE](LICENSE) for details.
