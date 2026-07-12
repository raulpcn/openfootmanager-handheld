<div align="center">

![Openfoot logo](images/openfootlogo.svg)

[![License: GPL v3](https://img.shields.io/github/license/raulpcn/openfootmanager-handheld
)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://shields.io/badge/-Rust-FF4500?style=flat&logo=rust)](https://www.rust-lang.org/)
[![Tauri](https://shields.io/badge/-Tauri-2E8B57?style=flat&logo=tauri)](https://tauri.app/)
[![React](https://shields.io/badge/-React-1434A4?style=flat&logo=react)](https://react.dev/)
[![CodeRabbit Pull Request Reviews](https://img.shields.io/coderabbit/prs/github/raulpcn/openfootmanager-handheld?utm_source=oss&utm_medium=github&utm_campaign=raulpcn%2Fopenfootmanager-handheld&labelColor=171717&color=FF570A&label=CodeRabbit+Reviews)](https://coderabbit.ai)
[![Maintenance](https://img.shields.io/badge/Maintained%3F-yes-green.svg)](https://github.com/raulpcn/openfootmanager-handheld/graphs/commit-activity)
[![Last commit](https://img.shields.io/github/last-commit/raulpcn/openfootmanager-handheld)](https://github.com/raulpcn/openfootmanager-handheld/commits)

**Openfoot Manager Handheld — a RG35XX H-focused build of Openfoot Manager**

[Features](#features) • [Screenshots](#screenshots) • [Installation](#installation--development) • [Contributing](#contributing) • [License](#license)

Repository: https://github.com/raulpcn/openfootmanager-handheld

</div>

---

**Openfoot Manager Handheld** is a free and open source football/soccer manager game fork, focused on the RG35XX H handheld experience and licensed under the [GPLv3](LICENSE.md), inspired by the famous franchise Football Manager&trade;.

## FEATURES

- **Text-based match simulation** with event-driven commentary and score progression.
- **Full squad management** for roles, depth planning, and player development decisions.
- **Transfer and contract workflows** to buy, sell, and negotiate player moves.
- **Training and staff systems** to improve performance through coaching and planning.
- **Dynamic inbox and news generation** that keeps you updated on club and world events.
- **Scouting support** for discovering talent and evaluating future signings.
- **Persistent game data** backed by SQLite for local saves and progression.
- **Modern desktop app experience** built with Tauri + React for speed and low overhead.
- **Multi-language support** with i18n foundations and community translation growth.
- **Free and open source** under GPLv3, with community-driven development.

## SCREENSHOTS

Click any image to open the full-size version.

<a href="images/screenshots/inbox.png"><img src="images/screenshots/inbox.png" alt="Inbox screen" width="220" /></a>
<a href="images/screenshots/news.png"><img src="images/screenshots/news.png" alt="News screen" width="220" /></a>
<a href="images/screenshots/manage_squad.png"><img src="images/screenshots/manage_squad.png" alt="Manage squad screen" width="220" /></a>

<a href="images/screenshots/matchlive.png"><img src="images/screenshots/matchlive.png" alt="Match live screen" width="220" /></a>
<a href="images/screenshots/training.png"><img src="images/screenshots/training.png" alt="Training screen" width="220" /></a>
<a href="images/screenshots/playertalk.png"><img src="images/screenshots/playertalk.png" alt="Player talk screen" width="220" /></a>

<a href="images/screenshots/presstalk.png"><img src="images/screenshots/presstalk.png" alt="Press talk screen" width="220" /></a>

## ARCHITECTURE

Openfoot Manager Handheld is built using modern web technologies:

- **Rust**: Blazing-fast backend for the Match Simulation Engine and Game State.
- **Tauri**: Lightweight desktop application shell.
- **React + TypeScript + TailwindCSS**: A highly responsive frontend interface.
- **SQLite**: Local persistence for game saves.

## INSTALLATION & DEVELOPMENT

This project tracks Openfoot Manager for handheld use. To build and run this fork, you need to install standard tools for Rust, Node, and Tauri development:

1. Install **Rust** (via `rustup`)
2. Install **Node.js** (v18+)
3. Install Tauri dependencies for your specific OS (see the [Tauri Prerequisites Guide](https://v2.tauri.app/start/prerequisites/))

Clone the repository and install dependencies:

```bash
git clone https://github.com/raulpcn/openfootmanager-handheld.git
cd openfootmanager-handheld
npm install
```

Run the development desktop app:

```bash
npm run tauri dev
```

## CONTRIBUTING

Contributions are welcome. For full guidelines, read [CONTRIBUTING](CONTRIBUTING.md).

If you want to discuss ideas or share feedback, open an issue in this repository.

Quick contribution checklist:

1. Open an Issue first for bugs, enhancements, or larger feature ideas.
2. Work from a feature branch and open Pull Requests targeting the main development branch.
3. Run tests before submitting:

```bash
npm test
cd src-tauri
cargo test --workspace
```

## LICENSE

    Openfoot Manager - A free and open source soccer management game
    Copyright (C) 2020-2026  Pedrenrique G. Guimarães

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.
    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.

Check [LICENSE](LICENSE.md) for more information.
