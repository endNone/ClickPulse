# frozen_string_literal: true

cask "clickpulse" do
  arch arm: "arm64", intel: "x64"

  version "0.3.1"
  sha256 arm:   "20ef66960cbaa024079dd95649df1785fee6fe86e68f68ac00729eaa25464474",
         intel: "818b87501a0282da9016db8d00e61f98a57ce67422a355c6b3c647a23ad2957e"

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
