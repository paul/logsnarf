# frozen_string_literal: true

require 'dry/events/publisher'
module Logsnarf
  UNIT = :microsecond
  MULT = 1_000_000
  MONOTONIC_OFFSET = Time.now.to_r * MULT - Process.clock_gettime(Process::CLOCK_MONOTONIC_RAW, UNIT)

  class Monitor
    include Dry::Events::Publisher["Logsnarf::Monitor"]

    attr_reader :id

    def initialize(id)
      @id = id
    end

    def start(event_id, payload)
      instrument(event_id, payload)
    end

    def stop(event_id, payload)
      instrument(event_id, payload)
    end

    def instrument(event_id, payload)
      payload[:start] = current
      result = yield payload if block_given?
    rescue Exception => e
      payload[:exception] = e
      raise
    ensure
      payload[:finish] = current
      process(event_id, payload) do |event, listener|
        listener.call(event.payload(payload))
      end

      result
    end

    private

    def self.current
      Process.clock_gettime(Process::CLOCK_MONOTONIC_RAW, :microsecond)
    end

    def current
      self.class.current
    end
  end

  class ::Dry::Events::Event
    def start
      @start ||= Time.at((Logsnarf::MONOTONIC_OFFSET + payload[:start]) / MULT)
    end

    def finish
      @finish ||= Time.at((Logsnarf::MONOTONIC_OFFSET + payload[:finish]) / MULT)
    end

    def duration
      @duration ||= ((payload[:finish] - payload[:start]).to_r / MULT).to_f
    end

    def duration_ms
      @duration_ms ||= duration * 1000
    end
  end
end
