# frozen_string_literal: true

require "async"
require "async/http/internet"
module Logsnarf
  class Writer
    MAX_QUEUE_SIZE = 1000
    MAX_DELAY = 5

    RequestError = Class.new(StandardError)

    def initialize(adapter:)
      @adapter = adapter

      @internet = Async::HTTP::Internet.new
      @metrics = []
      @last_send = now
      @semaphore = Async::Semaphore.new
      @task = Async::Task.current

      at_exit { send }
      setup_timer!
    end

    def push(metrics)
      @semaphore.acquire do
        @metrics.concat(metrics)
      end

      send if time_to_send?
    end

    def time_to_send?
      !@metrics.empty? &&
        @metrics.length > MAX_QUEUE_SIZE || (now - @last_send) > MAX_DELAY
    end

    def send
      metrics_to_send = nil
      @semaphore.acquire do
        metrics_to_send = @metrics.dup
        @metrics = []
        @last_send = now
      end

      return if metrics_to_send.empty?

      @task.async do
        @adapter.logger.info "sending #{metrics_to_send.size} metrics"
        @adapter.publish(metrics_to_send)
      end
    end

    def post(url, headers, body)
      @internet.post(url, headers, body)
    end

    def stop
      send
    ensure
      @internet.close
    end

    private

    def setup_timer!
      @timer = @task.async do |task|
        loop do
          send if time_to_send?
          task.sleep MAX_DELAY
        end
      end
    end

    def now
      Process.clock_gettime(Process::CLOCK_REALTIME)
    end
  end
end
