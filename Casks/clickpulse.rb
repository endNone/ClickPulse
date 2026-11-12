# frozen_string_literal: true

cask "clickpulse" do
  arch arm: "arm64", intel: "x64"

  version "0.2.1"
  sha256 arm:   "b82831ea5285e54a2b53afbad5e0e7af297bac2c134ae11b841a6b1d0f86721e",
         intel: "06298bc6138b7d947dfe81f161d5f4c8be1ce67e524dff71b9e67d75e003fd00"

  url "https://github.com/endNone/ClickPulse/releases/download/v#{version}/ClickPulse-#{version}-macos-#{arch}.dmg"
  name "ClickPulse"
  desc "Local keyboard and mouse activity statistics"
  homepage "https://github.com/endNone/ClickPulse"

  livecheck do
    url :url
    strategy :github_latest
  end

  auto_updates true
  depends_on macos: ">= :monterey"

  app "ClickPulse.app"

  caveats <<~EOS
    ClickPulse is not notarized. macOS may require Open Anyway on first launch.
    Enable Input Monitoring in System Settings to start recording.
    Updates may require refreshing the existing Input Monitoring permission.
    Installation guide: https://github.com/endNone/ClickPulse/blob/main/docs/install-macos.md
  EOS
end
