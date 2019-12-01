# frozen_string_literal: true

require "import"

module Logsnarf::Subscribers
  class LogSubscriber
    include Import[:logger]

    def on_client_write_metrics(event)
      metrics, body = event.payload.values_at(:metrics, :body)
      logger.info "Wrote %d metrics (%d bytes) in %0.2fms" % [metrics.size, body.bytesize, event.duration_ms]
    end

    def on_loader_load(event)
      account, bytes, lines, metrics = event.payload.values_at(:account, :bytes, :lines, :metrics)
      logger
        .with(name: account)
        .info "Received %db (%d lines) which parsed to %d metrics (%0.2fms)" % [bytes, lines, metrics.size, event.duration_ms]
    end

    Logsnarf::App[:instrumenter].subscribe(new)
  end
end
