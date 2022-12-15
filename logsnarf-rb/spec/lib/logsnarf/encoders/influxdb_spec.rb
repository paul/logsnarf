# frozen_string_literal: true

require "spec_helper"

require "logsnarf/decoder"
require "logsnarf/encoders/influxdb"

RSpec.describe Logsnarf::Encoders::Influxdb do
  let(:tags) { { "source" => "web.1" } }
  let(:now) { Time.now }
  let(:metric) { Logsnarf::Metric.new(name: "example", tags: tags, values: { value: 42 }, timestamp: now) }

  let(:encoder) { described_class::Encoder.new(metric) }

  it "should split the source into type and idx" do
    expect(encoder.tags).to eq("source" => "web.1", "type" => "web", "idx" => "1")
  end

  it "should use the source dyno number as a field value" do
    expect(encoder.fields).to eq("value" => "42i", "idx" => "1i")
  end

  describe "#to_s" do
    it "should generate an influxdb line protocol string" do
      expect(encoder.to_s)
        .to eq("example,source=web.1,type=web,idx=1 value=42i,idx=1i #{(now.to_f * 1_000_000).to_i}")
    end
  end
end
