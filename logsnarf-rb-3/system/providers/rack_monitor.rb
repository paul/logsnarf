# frozen_string_literal: true

App.register_provider :rack_monitor do
  prepare do
    require "dry/monitor/rack/middleware"
  end

  start do
    register(:rack_monitor, Dry::Monitor::Rack::Middleware.new(App[:notifications]))
  end
end
