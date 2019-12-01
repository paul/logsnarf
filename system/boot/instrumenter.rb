# frozen_string_literal: true

Logsnarf::App.boot(:instrumenter) do
  init do
    # require "dry-monitor"
    require "logsnarf/monitor"
  end

  start do
    instrumenter = Logsnarf::Monitor.new(:logsnarf)
    instrumenter.register_event("client.write_metrics")
    instrumenter.register_event("loader.load")

    register(:instrumenter, instrumenter)
  end
end
