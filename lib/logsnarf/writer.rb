# frozen_string_literal: true

require "raven"
require "async"
require "async/http/internet"
module Logsnarf
  class Writer
    MAX_QUEUE_SIZE = 1000
    MAX_DELAY = 5

    RequestError = Class.new(StandardError)

    def initialize(adapter:)
      @adapter = adapter
      @logger = adapter.logger
      logger.debug "Adapter initialized"

      @internet = Async::HTTP::Internet.new
      @metrics = []
      @last_send = now
      @semaphore = Async::Semaphore.new
      @task = Async::Task.current

      at_exit { stop }
      # setup_timer!
    end

    def push(metrics)
      @semaphore.acquire do
        @metrics.concat(metrics)
      end

      if time_to_send?
        send
      else
        start_timer
      end
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

      disable_timer

      @task.async do
        logger.info "sending #{metrics_to_send.size} metrics"
        @adapter.publish(metrics_to_send)
      rescue StandardError => e
        extra = { creds: @adapter.creds }
        if e.respond_to?(:response)
          response = e.response
          extra[:response] = {
            status: response.status,
            headers: response.headers.to_h,
            body: response.body
          }
        end
        extra[:request] = e.request if e.respond_to?(:request)

        Raven.capture_exception(e, extra: extra)
        raise
      end
    end

    def post(url, headers, body)
      @internet.post(url, headers, body)
    end

    def stop
      logger.debug "Adapter stopping"
      send
    ensure
      disable_timer
      @internet.close
    end

    private

    attr_reader :logger

    def start_timer
      @timer ||= @task.async do |task|
        logger.debug "timer started"
        task.sleep MAX_DELAY
        logger.debug "timer elapsed"
        send if time_to_send?
      end
    end

    def disable_timer
      logger.debug "timer stopped"
      @timer&.stop
    end

    def now
      Process.clock_gettime(Process::CLOCK_REALTIME)
    end
  end
end
