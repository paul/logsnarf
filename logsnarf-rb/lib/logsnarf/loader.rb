# frozen_string_literal: true

require "import"

module Logsnarf
  class Loader
    include Import[:logger, :instrumenter]
    attr_reader :adapter_store

    Error = Class.new(StandardError)
    class ParseError < Error
      def initialize(log_data, metric)
        @log_data, @metric = log_data, metric
        super <<~MSG
          Decoder recognized log line, but no data was extracted.
            #{log_data.line}
            #{metric.inspect}
        MSG
      end

      def raven_context
        {
          log_data: log_data,
          metric: metric
        }
      end
    end

    def initialize(credentials_store:, **imports)
      super(**imports)
      @credentials_store = credentials_store
      @adapter_store = LruRedux::TTL::ThreadSafeCache.new(1000, 15 * 60)
    end

    def load(token, io)
      creds = @credentials_store.fetch(token)
      raise AuthError, token if creds.nil?

      adapter = @adapter_store.getset(token) do
        Adapter[creds.type].new(creds)
      end

      text = io.read
      metrics = nil
      instrumenter.instrument("loader.load", lines: text.lines.size, bytes: text.bytes.size, account: creds["name"], metrics: []) do |payload|
        metrics = parse(text)
        payload[:metrics] = metrics

        adapter.write_metrics(metrics) unless metrics.empty?
      end
    end

    def parse(text)
      metrics = []
      parser = Parser.new(text)
      parser.each_metric do |log_data|
        decoder = DECODERS.detect { |dec| dec.valid?(log_data) }
        next unless decoder

        metric = decoder.new(log_data).call
        raise ParseError.new(log_data, metric) unless metric.complete?

        metrics << metric
      end
      metrics
    end
  end
end
