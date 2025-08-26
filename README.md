# qass

*For the reasonably paranoid.*

`qass` is a simple offline password manager that stores all logins in a human-readable and freely editable YAML file. The passwords are encrypted using [AES-GCM-SIV](https://docs.rs/aes-gcm-siv/latest/aes_gcm_siv/), and do not linger in memory. Retrieved passwords are not put on clipboards or anywhere else, but are typed directly using simulated keystrokes.

## Features

- **CLI and GUI**: The CLI exposes all capabilities of `qass` through a straightforward API. The GUI provides an ergonomic way to retrieve passwords, but is entirely optional.

- **Hierarchical Organization:** Organize passwords in a tree structure with path-based access. Password salts are kept in a separate YAML file to mitigate the impact of the primary store's accidental exposure.

- **Auto-typing:** Type passwords directly into applications, with active user confirmation to prevent accidental exposure.

- **CSV Import:** Import logins from CSV files exported from browsers or other password managers.

- **Offline:** Designed to work entirely offline, keeping your logins under your control. The password store is a directory of plain YAML files that can be trivially backed up.

- **Simple**: The CLI and internals are 740 lines of Rust in total, comparable to the well-known and loved [pass](https://git.zx2c4.com/password-store/about/). The GUI is another 360 lines of Rust. This simplicity enables thorough audits of the codebase in a short time. In fact, I implore users to do so before trusting any security-critical software of such impact.

## Installation

### Via Cargo

```bash
cargo install qass
```

Without GUI:
```
cargo install qass --features headless
```

### Prebuilt Binaries

Download prebuilt binaries from the [Releases page](https://github.com/boralg/qass/releases).

### From Source

Clone the repository:

```bash
git clone https://github.com/boralg/qass.git
cd qass
cargo build --release
```

If you use Nix, simply build using:

```bash
nix build
```

## Usage

### Initialize the Password Store

```bash
qass init
```

This creates the `~/.qass` directory. The primary store is `logins.yaml`. This can contain arbitrarily nested trees of login data. Leaves require a `username` and `password` field, but extra data can also be included. Paths down the tree are joined with `/` in the CLI and GUI.

Each encrypted password has a salt (and nonce) associated in `salts.yaml`. [Hidden paths](#hiding-sensitive-logins) are stored in `hidden.yaml`.

### Add a New Login

```bash
qass add github.com/myusername myusername
# You'll be prompted for the password and master password
```

### Retrieve and Type a Password

```bash
qass type github.com/myusername
# Enter master password, focus the target field, then press CONTROL (within a timeout interval) to type the password
```

### List All Logins

```bash
qass list
```

### Import from CSV

```bash
qass import passwords.csv
# The CSV must have 'url', 'username', and 'password' columns
```

### Hiding Sensitive Logins

Hide logins behind an additional layer of encryption that hides all fields and pathnames too:

```bash
qass hide banking
# All logins under the banking path will be hidden
```

Access a hidden login:

```bash
qass type-hidden banking/chase-bank/user
# Requires both the [master password used for hiding] and the [master password used for encrypting the password], in this order
```


Unhide previously hidden logins:

```bash
qass unhide banking
```

### Syncing and Unlocking

Encrypt cleartext logins (e.g. after adding them by hand to `logins.yaml`, or after using `unlock`):

```bash
qass sync /
```

Decrypt logins for cleartext access:

```bash
qass unlock /
```

### GUI

This is an [`egui`](https://docs.rs/egui/latest/egui/) application that allows for quick searches among your stored logins, then retrieving passwords. It comes with numerous measures built in to increase the security of not just the passwords, but the login paths as well. 

The master password is handled with a custom-made widget to make sure it's never copied internally, and is zeroed out in memory after use. <br>
The login search is aided by auto-complete, but it's entirely opt-in (at multiple steps), so that you can control which pathnames can external observers see.

To use the GUI:

```
qass gui
```

1. Enter the path to the desired login. You may press `Tab` at any point to see suggestions of all paths that start with your input. Only a few are shown at a time, but can be scrolled with the arrow keys. To accept a suggestion, press `Tab` or `Enter`. To limit exposure of paths, suggestions only complete the current segment of a path. To go from there, you can show suggestions again with `Tab`.
2. When you've entered the desired login path, press `Enter`, then enter the master password. To not expose its length, `qass` doesn't display a password field, but you can still type it in normally.
3. Once you've entered the master password, press `Enter`. At this point - just like the CLI - the GUI will prompt you to focus the field you want to enter the password into. After the confirmation keypress, the decrypted password will be automatically typed by `qass`.

## Security Considerations

- Master passwords are never stored.
- No permanent secret is derived from master passwords. Hence, different logins can be encrypted with different master passwords, without external observers being able to tell.
- Passwords are encrypted with [AES-GCM-SIV](https://docs.rs/aes-gcm-siv/latest/aes_gcm_siv/).
- Key derivation uses [Argon2](https://docs.rs/argon2/latest/argon2).
- Short passwords are padded to 32 bytes before encryption so the ciphertext doesn't expose their lengths.
- Sensitive data is zeroed from memory when no longer needed.
- All operations are performed locally.
- The GUI exposes minimal information of the store during its operation.

## Contributing

Contributions to `qass` are welcome! If you have suggestions or encounter issues/vulnerabilities, please open an issue or submit a pull request. Feature requests will be considered, but I will be selective to maintain the project's simplicity and security.