# frozen_string_literal: true

class Extractor
  include Import[:parser, :decoder, :logger]
  include Dry::Monads[:result]

  def extract(io)
    metrics = []
    parser.parse(io) do |log_data|
      metric = decoder.decode(log_data)
      metrics << metric if metric
    end
    Success(metrics)
  end
end
