# VRCManager - Automated VRChat Group Management Utility

![License](https://img.shields.io/badge/License-CC%20BY--NC--SA%204.0-lightgrey.svg)

Automate group moderation and management tasks in VRChat through real-time log monitoring.

## Key Features

- 🕵️‍♂️ Real-time monitoring of VRChat log files
- 🔒 Secure authentication with 2FA support
- 🧾 Persistent cookie storage for seamless logins
- 🚫 Automatic group banning of users with prohibited avatars
- 📬 Automatic group invites
- ⚙️ Customizable through a simple configuration file

## How Automatic Bans Work

1. Monitors VRChat's latest log file for player join events
2. Extracts user IDs from log entries
3. Fetches user's current avatar via VRChat API
4. Checks avatar against your blocklist (`avatars.txt`)
5. Automatically bans users with prohibited avatars from your group

## Setup & Configuration

### 1. Clone the repository
```bash
git clone https://github.com/RavMda/vrc-manager.git
cd vrc-manager
```

### 2. Create config file (`config.toml`)
```toml
auto_invite = true
auto_ban = false
group_id = "grp_f0db2b50-9440-4e8f-bd09-75870a423dd7" #required
avatars_file = "avatars.txt" # optional
custom_log_dir = "/home/whatever/something/vrchat" # optional
```

### 3. Create avatar blocklist (if automatic banning is used)
Modify existing `avatars.txt` (or your custom-named file) with one avatar ID per line:
```
avtr_12345678-90ab-cdef-1234-567890abcdef
avtr_aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee
```

### 4. Build the project
```bash
cargo build --release
```

## Usage

```bash
# Run the application
cargo run --release

# Or run the compiled binary
./target/release/vrc-autoban # .exe if on Windows
```
## Important Notes
- **Rate Limits**: VRChat API has rate limits - use responsibly
- **Log Format**: Depends on VRChat's current log format (may need updates)
- **Long Reaction Times**: May take a few seconds to process log entries, and the player won't get kicked instantly because of limitations

## Contributing 🤝
1. Fork the repository
2. Create your feature branch (`git checkout -b feature/fooBar`)
3. Commit your changes (`git commit -am 'Add some fooBar'`)
4. Push to the branch (`git push origin feature/fooBar`)
5. Create a new Pull Request
