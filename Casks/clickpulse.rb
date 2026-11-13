# frozen_string_literal: true

cask "clickpulse" do
  arch arm: "arm64", intel: "x64"

  version "0.3.0"
  sha256 arm:   "4e2e50a24436f0fa3bf6add2b4dc081146c53ec9d4cf4420f7809836ab05bc5e",
         intel: "55d6964924c47b28caf34d7429766d32d112114241b4f9dda31f9ea25e456783"

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

  caveats <<~EOS
    ClickPulse is not notarized. macOS may require Open Anyway on first launch.
    Enable Input Monitoring in System Settings to start recording.
    Updates may require refreshing the existing Input Monitoring permission.
    Installation guide: https://github.com/endNone/ClickPulse/blob/main/docs/install-macos.md
  EOS
end
