# frozen_string_literal: true

require "spec_helper"

require "logsnarf/decoder"
require "logsnarf/encoders/influxdb"

RSpec.describe Logsnarf::Encoders::Influxdb do
  let(:tags) { { "source" => "web.1" } }
  let(:metric) { Logsnarf::Metric.new(name: "example", tags: tags, values: { value: 1 }, timestamp: Time.now) }

  let(:encoder) { described_class::Encoder.new(metric) }

  it "should split the source into type and idx" do
    expect(encoder.tags).to eq("source=web.1,type=web,idx=1")
  end
end
