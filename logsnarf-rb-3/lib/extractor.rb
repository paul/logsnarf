# frozen_string_literal: true

class Extractor
  include Import[:parser, :decoder, :notifications, :logger, :notifier]
  include Dry::Monads[:result]

  def extract(io)
    metrics = []
    notifications.instrument("extractor.extract", {}) do |payload|
      lines, bytes = parser.parse(io) do |log_data|
        metric = decoder.decode(log_data)
        next unless metric

        if metric.complete?
          metrics << metric
        else
          logger.info self, "Found a line I expected to decode, but didn't get a complete metric\n#{log_data.inspect}\n#{metric.inspect}"
        end
      end
      payload.merge!(lines:, bytes:, metrics:)
    end
    Success(metrics)
  end
end
