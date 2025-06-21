# VRCAutoBan - Automatic VRChat Group Ban Utility

![License](https://img.shields.io/badge/License-CC%20BY--NC--SA%204.0-lightgrey.svg)

Automatically ban VRChat players who use prohibited avatars by monitoring game logs in real-time.

## Key Features

- 🕵️‍♂️ Real-time monitoring of VRChat log files
- 🔒 Secure authentication with 2FA support
- 🧾 Persistent cookie storage for seamless logins
- 🚫 Automatic group banning of users with prohibited avatars
- ⚙️ Configurable through environment variables

## How It Works

1. Monitors VRChat's latest log file for player join events
2. Extracts user IDs from log entries
3. Fetches user's current avatar via VRChat API
4. Checks avatar against your blocklist (`avatars.txt`)
5. Automatically bans users with prohibited avatars from your group

## Setup & Configuration

### 1. Clone the repository
```bash
git clone https://github.com/yourusername/vrc-autoban.git
cd vrc-autoban
```

### 2. Create environment file (`.env`)
```env
# Required
GROUP_ID=grp_00000000-0000-0000-0000-000000000000

# Optional (defaults to avatars.txt)
AVATAR_FILE=custom_blocklist.txt

# Optional log directory override
CUSTOM_LOG_DIR=/home/whatever/something/vrchat/logs
```

### 3. Create avatar blocklist
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
./target/release/vrc-autoban
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
