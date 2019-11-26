# frozen_string_literal: true

module Logsnarf
  class Loader
    attr_reader :logger, :instrumenter, :adapter_store

    def initialize(logger:, instrumenter:, credentials_store:)
      @logger, @instrumenter = logger, instrumenter
      @credentials_store = credentials_store
      @adapter_store = LruRedux::TTL::ThreadSafeCache.new(1000, 15 * 60)
    end

    def load(token, io)
      creds = @credentials_store.fetch(token)
      raise AuthError, token if creds.nil?

      adapter = @adapter_store.getset(token) do
        Adapter[creds.type].new(creds, logger: logger, instrumenter: instrumenter)
      end

      text = io.read
      metrics = nil
      instrumenter.instrument("load", lines: text.lines.size, bytes: text.bytes.size, account: creds["name"]) do |payload|
        payload.measure("parse") do
          metrics = parse(text)
        end

        payload[:metrics] = metrics.size

        adapter.write_metrics(metrics) unless metrics.empty?
      end
    end

    def parse(text)
      metrics = []
      parser = Parser.new(text)
      parser.each_metric do |log_data|
        decoder = DECODERS.detect { |dec| dec.valid?(log_data) }&.new(log_data)
        metrics << decoder if decoder
      end
      metrics
    end
  end
end
