# frozen_string_literal: true

require "import"

module Logsnarf
  module Adapter
    class InfluxdbV1
      include Import[:logger, :instrumenter]

      def initialize(creds, **imports)
        super(**imports)
        @creds = creds
        @logger = logger.with(name: "influxdb_v1 #{creds['name']}")
        @url = @creds.dig("credentials", "influxdb_url")
        @client = Logsnarf::Clients::InfluxdbV1.new(url: @url, logger: @logger)
        @buffer = MetricsBuffer.new
      end

      def stop
        logger.debug "Adapter stopping"
        flush
        @client.stop
      end

      def write_metrics(metrics)
        @buffer.push(metrics)
        start_timer
        try_flush
      end

      private

      attr_reader :creds

      def try_flush
        if @buffer.flushable?
          flush
          @timer&.stop
        end
      end

      def flush
        @client.write(@buffer.flush) unless @buffer.empty?
      end

      def start_timer
        @timer = nil if @timer&.finished?
        @timer ||= Async do |task|
          logger.debug "Timer started"
          task.sleep 10
          logger.debug "Timer expired, flushing"
          flush
        end
      end

      require "async/clock"
      class MetricsBuffer
        SIZE_LIMIT = 1000
        DURATION_LIMIT = 5

        def initialize(size_limit: SIZE_LIMIT, duration_limit: DURATION_LIMIT)
          @size_limit, @duration_limit = size_limit, duration_limit
          @last_flush = now
          @buffer = []
        end

        # Adds items to the buffer
        def push(items)
          @buffer.concat(items)
        end

        # Empties the buffer, returning an array of all the items that were in
        # the buffer
        def flush
          values = @buffer.dup
          @buffer.clear
          @last_flush = now
          values
        end

        def flushable?
          full? || elapsed?
        end

        def empty?
          @buffer.empty?
        end

        private

        attr_reader :size_limit, :duration_limit, :last_flush

        def full?
          @buffer.size > size_limit
        end

        def elapsed?
          now - last_flush > duration_limit
        end

        def now
          Async::Clock.now
        end
      end
    end
  end
end
