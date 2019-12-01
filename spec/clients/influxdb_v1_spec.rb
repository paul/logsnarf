# frozen_string_literal: true

require "spec_helper"

RSpec.describe Logsnarf::Clients::InfluxdbV1 do
  let(:now) { Time.now }
  let(:metrics) do
    [
      Logsnarf::Metric.new(name: "test_data", tags: { my: :tag }, values: { value: 1234 }, timestamp: now)
    ]
  end

  let(:response) { OpenStruct.new(status: 200) }
  let(:http) { instance_spy("internet", post: response) }
  let(:url) {  "http://influxdb.example/test" }
  subject(:client) { described_class.new(url: url, http: http) }

  it "should post measurements to influx" do
    client.write(metrics)
    expect(http).to have_received(:post).with(
      "http://influxdb.example/write?db=test&precision=u", [], kind_of(String)
    )
  end

  describe "authorization" do
    let(:url) { "http://AzureDiamond:hunter2@influxdb.example/test" }
    it "should extract userinfo from the url as an Authorization header" do
      client.write(metrics)
      expect(http).to have_received(:post).with(
        "http://influxdb.example/write?db=test&precision=u",
        [["Authorization", "Basic QXp1cmVEaWFtb25kOmh1bnRlcjI="]],
        kind_of(String)
      )
    end
  end

  describe "post body" do
    it "should encode the metrics as influx measurement data" do
      client.write(metrics)
      expect(http).to have_received(:post).with(
        "http://influxdb.example/write?db=test&precision=u",
        [],
        "test_data,my=tag value=1234i #{(now.to_r * 1_000_000).to_i}"
      )
    end
  end

  describe "instrumentation" do
    let(:listener) { instance_spy("listener", on_client_write_metrics: true, on_loader_load: true) }
    before { Logsnarf::App[:instrumenter].subscribe(listener) }
    after { Logsnarf::App[:instrumenter].unsubscribe(listener) }

    it "should report instrument the request" do
      client.write(metrics)
      expect(listener).to have_received(:on_client_write_metrics)
        .with(having_attributes(payload: hash_including(
          metrics: metrics,
          body: kind_of(String),
          response: response
        )))
    end

    context "when the request raises an exception" do
      let(:listener) { double("listener", on_client_write_metrics: true) }
      MyError = Class.new(StandardError)
      it "should record the error in the payload" do
        allow(http).to receive(:post).and_raise(MyError)
        allow(listener).to receive(:on_client_write_metrics) { |event| ap event }
        expect { client.write(metrics).wait }.to raise_error(MyError)
        expect(listener).to have_received(:on_client_write_metrics)
          .with(having_attributes(payload: hash_including(
            exception: kind_of(MyError)
          )))
      end
    end
  end
end
