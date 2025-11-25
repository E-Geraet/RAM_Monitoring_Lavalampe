#!/bin/bash
set -e

echo "RAM Lava Lamp Installer"
echo "=========================="

# Gehe zum Projekt-Verzeichnis
cd "$(dirname "$0")"

echo "Kompiliere Projekt (Release-Modus)..."
cargo build --release

echo "Erstelle Zielverzeichnis..."
mkdir -p ~/.local/bin
mkdir -p ~/.local/share/ram-lavalampe

echo "Kopiere ausführbare Datei..."
cp target/release/ram-lavalampe ~/.local/bin/

echo "Kopiere Assets..."
cp -r assets ~/.local/share/ram-lavalampe/

echo "Installation abgeschlossen!"
echo ""
echo "Die Lavalampe kann jetzt von überall gestartet werden:"
echo "  ram-lavalampe"
echo ""
echo "Hinweis: Stelle sicher, dass ~/.local/bin in deinem PATH ist."
echo "Falls nicht, füge folgende Zeile zu ~/.bashrc hinzu:"
echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""