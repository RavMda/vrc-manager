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

## Download Pre-Built Binaries

Pre-built binaries are available for download on the [Releases page](https://github.com/RavMda/vrc-manager/releases).

## Configuration

### 1. Create config file (`config.toml`)
```toml
group_id = "grp_f0db2b50-9440-4e8f-bd09-75870a423dd7"
avatars_file = "avatars.txt"                          # optional
custom_log_dir = "/home/whatever/something/vrchat"    # optional

[auto_invite]
enabled = true
delay_min = 240  # seconds
delay_max = 360 # seconds

[auto_ban]
enabled = true
log_avatar_id = true
```

### 2. Create avatar file id blocklist (if automatic banning is used)
Modify existing `avatars.txt` (or your custom-named file) with one avatar file ID per line:
```
file_12345678-90ab-cdef-1234-567890abcdef
file_aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee
```

## Building from Source

```bash
# Clone the repository
git clone https://github.com/RavMda/vrc-manager.git    
cd vrc-manager

# Build the project
cargo build --release

# The binary will be available at "target/release" folder

# Or, you can just simply run it
cargo run --release
```

## Important Notes
- **Rate Limits**: VRChat API has rate limits - use responsibly
- **Log Format**: Depends on VRChat's current log format (may need updates)

## Contributing 🤝
1. Fork the repository
2. Create your feature branch (`git checkout -b feature/fooBar`)
3. Commit your changes (`git commit -am 'Add some fooBar'`)
4. Push to the branch (`git push origin feature/fooBar`)
5. Create a new Pull Request
