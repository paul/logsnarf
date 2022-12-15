# frozen_string_literal: true

Logsnarf::App.boot :rack_logger do
  init do
    require "dry/monitor/rack/logger"
  end

  settings do
    key :rack_logger, Types::Any
  end

  configure do |config|
    config.rack_logger = Dry::Monitor::Rack::Logger.new(Logsnarf::App[:logger])
  end

  start do
    config.rack_logger.attach(Logsnarf::App[:rack_monitor])
  end
end
