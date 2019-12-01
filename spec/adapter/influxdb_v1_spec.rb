# frozen_string_literal: true

require "spec_helper"

RSpec.describe Logsnarf::Adapter::InfluxdbV1 do
  let(:now) { Time.now }
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
  let(:adapter) { Logsnarf::Adapter::InfluxdbV1.new(creds) }

  let(:metrics) do
    [
      Logsnarf::Metric.new(name: "test_data", tags: { my: :tag }, values: { value: 1234 }, timestamp: now)
    ]
  end

  it "should work" do
    adapter.write_metrics(metrics)
  end
end
