# frozen_string_literal: true

cask "clickpulse" do
  arch arm: "arm64", intel: "x64"

  version "0.3.2"
  sha256 arm:   "800a692ac7b595ae140c9f2fee6e273977d8ffa830ab07e6c774ea0181031724",
         intel: "ac3325dd3fd186e1168fed3879e9b8ebf9961b6c7c1cbb75c8dc11aa0a60aa8e"

  url "https://github.com/endNone/ClickPulse/releases/download/v#{version}/ClickPulse-#{version}-macos-#{arch}.dmg"
  name "ClickPulse"
  desc "Local keyboard and mouse activity statistics"
  homepage "https://github.com/endNone/ClickPulse"

  livecheck do
    url :url
    strategy :github_latest
  end

  auto_updates true
  depends_on macos: :monterey

  app "ClickPulse.app"

  uninstall launchctl: "app.clickpulse.desktop.login",
            quit:      "app.clickpulse.desktop"

  zap trash: [
    "~/Library/Application Support/ClickPulse",
    "~/Library/Caches/ClickPulse",
    "~/Library/LaunchAgents/app.clickpulse.desktop.login.plist",
    "~/Library/Logs/clickpulse.log*",
    "~/Library/Preferences/app.clickpulse.desktop.plist",
    "~/Library/WebKit/app.clickpulse.desktop",
  ]

  caveats <<~EOS
    ClickPulse is not notarized. macOS may require Open Anyway on first launch.
    Enable Input Monitoring in System Settings to start recording.
    Updates may require refreshing the existing Input Monitoring permission.
    Installation guide: https://github.com/endNone/ClickPulse/blob/main/docs/install-macos.md
  EOS
end
