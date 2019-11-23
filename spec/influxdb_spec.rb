# frozen_string_literal: true

require "spec_helper"
require "async/rspec"

RSpec.describe Logsnarf::Adapter::InfluxdbV1 do
  include_context Async::RSpec::Reactor
  let(:now) { Time.now }
  let(:data) { <<~TXT }
    245 <45>1 #{now.iso8601} d.475fd4b7-03da-4e45-8c89-5d8ac5fff61d heroku worker.1 - - source=worker.1 dyno=heroku.97268060.75eb7bb9-ab78-41be-9cc7-576eaad6dae7 sample#load_avg_1m=0.03 sample#load_avg_5m=0.04 sample#load_avg_15m=0.02
    377 <45>1 #{now.iso8601} d.475fd4b7-03da-4e45-8c89-5d8ac5fff61d heroku worker.1 - - source=worker.1 dyno=heroku.97268060.75eb7bb9-ab78-41be-9cc7-576eaad6dae7 sample#memory_total=318.20MB sample#memory_rss=304.71MB sample#memory_cache=13.48MB sample#memory_swap=0.00MB sample#memory_pgpgin=143756pages sample#memory_pgpgout=62297pages sample#memory_quota=512.00MB
  TXT

  let(:creds) do
    {
      "credentials" => {
        "influxdb_url" => "https://localhost:8087/logsnarf",
        "type" => "influxdb_v1"
      },
      "name" => "logsnarf local testing",
      "token" => "e0ff2e6751893dcd7fcb7a94d4535437"
    }
  end
  let(:adapter) { Logsnarf::Adapter::InfluxdbV1.new(creds, logger: Console.logger, instrumenter: nil) }

  let(:metrics) do
    metrics = []
    parser = Logsnarf::Parser.new(data)
    parser.each_metric do |log_data|
      decoder = Logsnarf::DECODERS.detect { |dec| dec.valid?(log_data) }&.new(log_data)
      metrics << decoder if decoder
    end
    metrics
  end

  it "should work" do
    Console.logger.debug!
    task = reactor.async do |_task|
      adapter.write_metrics(metrics)
      adapter.stop
    end
    task.wait
  end
end
