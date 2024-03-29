# frozen_string_literal: true

require "spec_helper"
RSpec.describe Logsnarf::Loader do
  class FakeInflux
    def initialize
      @metrics = []
    end

    def write_metric(metric)
      @metrics << metric
    end
  end
  let(:creds) {
    {
      "credentials" => {
        "influxdb_url" => "http://localhost:8086/logsnarf",
        "type" => "influxdb_v1"
      },
      "name" => "test"
    }
  }
  let(:credentials_store) { { token => Logsnarf::Credentials.new(creds) } }

  let(:token) { "credentials-token" }
  let(:now) { Time.now }
  let(:io) { StringIO.new(<<~TXT) }
    245 <45>1 #{now.iso8601} d.475fd4b7-03da-4e45-8c89-5d8ac5fff61d heroku worker.1 - - source=worker.1 dyno=heroku.97268060.75eb7bb9-ab78-41be-9cc7-576eaad6dae7 sample#load_avg_1m=0.03 sample#load_avg_5m=0.04 sample#load_avg_15m=0.02
    377 <45>1 #{now.iso8601} d.475fd4b7-03da-4e45-8c89-5d8ac5fff61d heroku worker.1 - - source=worker.1 dyno=heroku.97268060.75eb7bb9-ab78-41be-9cc7-576eaad6dae7 sample#memory_total=318.20MB sample#memory_rss=304.71MB sample#memory_cache=13.48MB sample#memory_swap=0.00MB sample#memory_pgpgin=143756pages sample#memory_pgpgout=62297pages sample#memory_quota=512.00MB
  TXT

  subject(:loader) { described_class.new(credentials_store: credentials_store) }

  it "should work" do
    loader.load(token, io)
  end

  context "when the log line looks like it matches a parser, but has no data" do
    let(:io) { StringIO.new(<<~TXT) }
      245 <45>1 #{now.iso8601} d.475fd4b7-03da-4e45-8c89-5d8ac5fff61d heroku heroku-redis - - source=MYREDIS
    TXT

    it "should raise a Loader::ParseError" do
      expect { loader.load(token, io) }
        .to raise_error(Logsnarf::Loader::ParseError)
    end
  end
end
