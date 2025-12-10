# Quest Passer

A lightweight, modern utility to help you complete Discord Quests by simulating game processes without installing the actual games.

![Quest Passer](https://img.shields.io/badge/Version-1.0.0-blue.svg)

## 🚀 Features

- **Auto-Detection**: Fetches the latest list of Quest-eligible games directly from Discord.
- **Quest Queue**: Create a playlist of games. The app will run each for 15 minutes and 30 seconds (configurable) and automatically move to the next.
- **Custom Games**: Add any Discord Application ID manually to support new or unlisted games.
- **Queue Estimator**: Real-time estimation of when your quest queue will finish.
- **Modern UI**: Sleek dark mode design with glassmorphism and smooth animations.

## ⚠️ Disclaimer & Terms of Service

### Educational Purpose Only
This project was created purely for **educational purposes** to demonstrate system programming, Win32 API integration in Rust, and modern React interface design.

### Liability
**By downloading or using this software, you agree that:**
1.  The creator (**nrj900**) and any contributors are **NOT responsible** for any consequences to your Discord account.
2.  You are using this software entirely at your own risk.
3.  We are not liable for any account suspensions, bans, warnings, or loss of access to Discord services that may result from the use of this tool.

### Terms of Service Compliance
Please respect Discord's platform.
- **Discord Terms of Service**: [https://discord.com/terms](https://discord.com/terms)
- **Discord Community Guidelines**: [https://discord.com/guidelines](https://discord.com/guidelines)

While this tool primarily simulates Rich Presence (RPC) activity—which is generally harmless—using automation tools to interact with the platform can potentially be flagged depending on Discord's evolving policies. **Use responsibly and moderately.**

---

## 🛠️ Installation & Usage

1.  **Download**: Get the latest `.exe` installer from the [Releases](https://github.com/NRJ900/Quest-Passer/releases) page.
2.  **Install**: Run the setup file.
3.  **Run**: Open **Quest Passer**.
4.  **Play**:
    *   **Search** for a game (e.g., "Genshin Impact").
    *   Click **Play** to start immediately.
    *   Click **Queue** to add it to your playlist.
5.  **Relax**: The app will notify you (beep) when the quest time (15m) is up!

## 🔧 Building from Source

Requirements: `Node.js`, `Rust`, `Cargo`.

```bash
# Install dependencies
npm install

# Run in Development Mode
npm run tauri dev

# Build for Windows
npm run tauri build
```
