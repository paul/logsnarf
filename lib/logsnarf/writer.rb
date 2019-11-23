# frozen_string_literal: true

require "raven"
require "async"
require "async/http/internet"
module Logsnarf
  class Writer
    MAX_QUEUE_SIZE = 1000
    MAX_DELAY = 30

    class RequestError < StandardError
      attr_reader :response, :request

      def initialize(response)
        @response = response
      end

      def message
        %{Request failed: #{response.status}\n#{response.body&.read}}
      end
    end

    def initialize(adapter:)
      @adapter = adapter
      @logger = adapter.logger
      logger.debug "Adapter initialized"

      @internet = Async::HTTP::Internet.new
      @metrics = []
      @last_send = now
      @semaphore = Async::Semaphore.new

      at_exit { stop }
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
      Async do
        metrics_to_send = nil
        disable_timer
        @semaphore.acquire do
          metrics_to_send = @metrics.dup
          @metrics = []
          @last_send = now
        end

        if !metrics_to_send.empty?

          logger.info "sending #{metrics_to_send.size} metrics"
          request = [@adapter.url, @adapter.headers, @adapter.encode(metrics_to_send)]
          logger.debug { " ==> #{request[0]} #{request[1].to_h.inspect}" }
          response = @internet.post(*request)
          logger.debug { " <== #{response.status} #{response.headers.to_h}" }
          raise RequestError, response unless (200..299).cover?(response.status)
        end
      rescue StandardError => e
        if e.is_a?(RequestError)
          extra = {
            request: request,
            response: response,
            creds: @adapter.creds,
            response_body: e&.response&.body&.read
          }
        end
        Raven.capture_exception(e, extra: extra || {})
        raise
      end
    end

    def stop
      logger.debug "Adapter stopping"
      send.wait
    ensure
      disable_timer
      @internet.close
    end

    private

    attr_reader :logger

    def start_timer
      @timer ||= Async do |task|
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
