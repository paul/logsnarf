# frozen_string_literal: true

module Logsnarf::Subscribers
  class InfluxSubscriber
    include Import[:influxdb]

    def on_loader_load(event)
      payload = event.payload
      values  = payload.slice(:bytes, :lines).merge(
        metrics: payload[:metrics].size,
        duration: event.duration
      )
      metric = Logsnarf::Metric.new(
        name: "loader.load",
        tags: payload.slice(:account),
        values: values,
        timestamp: event.start
      )
      influxdb.write([metric])
    end

    # Logsnarf::App[:instrumenter].subscribe(new)
  end
end
