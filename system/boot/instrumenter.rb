# frozen_string_literal: true

Logsnarf::App.boot(:instrumenter) do
  init do
    require "dry-monitor"
  end

  start do
    register(:instrumenter, Dry::Monitor::Notifications.new(:logsnarf))
  end
end
