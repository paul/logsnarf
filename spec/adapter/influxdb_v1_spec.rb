# frozen_string_literal: true

require "spec_helper"

RSpec.describe Logsnarf::Adapter::InfluxdbV1 do
  let(:now) { Time.now }
  let(:data) { <<~TXT }
    245 <45>1 #{now.iso8601} d.475fd4b7-03da-4e45-8c89-5d8ac5fff61d heroku worker.1 - - source=worker.1 dyno=heroku.97268060.75eb7bb9-ab78-41be-9cc7-576eaad6dae7 sample#load_avg_1m=0.03 sample#load_avg_5m=0.04 sample#load_avg_15m=0.02
    377 <45>1 #{now.iso8601} d.475fd4b7-03da-4e45-8c89-5d8ac5fff61d heroku worker.1 - - source=worker.1 dyno=heroku.97268060.75eb7bb9-ab78-41be-9cc7-576eaad6dae7 sample#memory_total=318.20MB sample#memory_rss=304.71MB sample#memory_cache=13.48MB sample#memory_swap=0.00MB sample#memory_pgpgin=143756pages sample#memory_pgpgout=62297pages sample#memory_quota=512.00MB
  TXT

  let(:creds) do
    {
      "credentials" => {
        "influxdb_url" => "http://localhost:8086/logsnarf",
        "type" => "influxdb_v1"
      },
      "name" => "logsnarf local testing",
      "token" => "e0ff2e6751893dcd7fcb7a94d4535437"
    }
  end
  let(:adapter) { Logsnarf::Adapter::InfluxdbV1.new(creds, logger: Console.logger, instrumenter: nil) }

  let(:metrics) do
    [
      Logsnarf::Metric.new(log_data: data, name: "test_data", tags: { my: :tag }, values: { value: 1234 }, timestamp: now)
    ]
  end

  it "should work" do
    adapter.write_metrics(metrics)
  end
end
