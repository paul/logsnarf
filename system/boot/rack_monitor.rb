# frozen_string_literal: true

Logsnarf::App.boot :rack_monitor do
  init do
    require "dry/monitor/rack/middleware"
  end

  start do
    register(:rack_monitor, Dry::Monitor::Rack::Middleware.new(Logsnarf::App[:notifications]))
  end
end
