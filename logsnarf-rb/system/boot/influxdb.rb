# frozen_string_literal: true

Logsnarf::App.boot(:influxdb) do
  init do
    require "logsnarf/clients/influxdb_v1"
  end

  settings do
    key :influxdb_url, Types::Strict::String
  end

  configure do |config|
    config.influxdb_url = ENV["INFLUXDB_URL"]
  end

  start do
    register(:influxdb, Logsnarf::Clients::InfluxdbV1.new(url: config.influxdb_url))
  end
end
