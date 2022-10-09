# frozen_string_literal: true

module Subscribers
  class Extractor
    include Import[:logger]

    def on_extractor_extract(event)
      bytes, lines, metrics, time = event.payload.values_at(:bytes, :lines, :metrics, :time)
      logger.info "Received %s (%d lines) which parsed to %d metrics (%dms)" %
                  [filesize(bytes), lines, metrics.size, time]
    end

    def on_client_write_metrics(event)
      client, metrics, body, time = event.payload.values_at(:client, :metrics, :body, :time)
      logger
        .with(name: client)
        .info "Wrote %d metrics (%s) in %dms" % [metrics.size, filesize(body.bytesize), time]
    end

    private

    def filesize(size)
      units = %w[B KiB MiB GiB TiB Pib EiB ZiB]

      return "0.0 B" if size == 0

      exp = (Math.log(size) / Math.log(1024)).to_i
      exp += 1 if size.to_f / 1024**exp >= 1024 - 0.05
      exp = units.size - 1 if exp > units.size - 1

      "%.1f %s" % [size.to_f / 1024**exp, units[exp]]
    end

    App[:notifications].subscribe(Subscribers::Extractor.new)
  end
end
