# frozen_string_literal: true

module Logsnarf
  class Instrumenter
    attr_reader :adapter

    def initialize(adapter:)
      @adapter = adapter
    end

    def instrument(name, payload = {}, &block)
      event = Event.new(name, payload)
      event.measure(&block)

      @adapter.write_metric(event)
    end

    require "benchmark"
    require "forwardable"
    class Event
      extend Forwardable

      delegate [:[], :[]=] => :@payload

      attr_reader :name, :timestamp

      def initialize(name, payload = {})
        @name, @payload = name, payload
        @timestamp = Process.clock_gettime(Process::CLOCK_REALTIME)
      end

      def measure(name = nil)
        out = nil
        time = Benchmark.measure do
          out = yield self
        end

        name = [name, "duration_s"].compact.join("_")

        @payload[name] = time.real
        out
      end

      def tags
        @payload.select { |_k, v| v.is_a?(String) }
      end

      def values
        @payload.select { |_k, v| v.is_a?(Numeric) }
      end
    end
  end

  class NullInstrumenter < Instrumenter
    def initialize(adapter: nil); end

    def instrument(name, payload = {}, &block)
      event = Event.new(name, payload)
      event.measure(&block)
    end
  end
end
