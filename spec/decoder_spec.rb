# frozen_string_literal: true

require "spec_helper"

require "logsnarf/decoder"

RSpec.describe Logsnarf::Decoder do
  let(:sample) { log_sample(fixture) }
  let(:log_data) { Logsnarf::Parser.new(sample).first_line }

  let(:decoder) { described_class.new(log_data) }
  subject(:metric) { decoder.call }

  shared_examples "Decoder" do
    it "should be valid on the data" do
      expect(described_class.valid?(log_data)).to be true
    end

    it "should not be valid on other data" do
      other_samples(fixture).each do |name, sample|
        data = Logsnarf::Parser.new(sample).first_line
        expect(described_class.valid?(data)).to be(false), "expected to not be valid on #{name}"
      end
    end

    specify { expect(metric.name).to      eq expected_name      }
    specify { expect(metric.timestamp).to eq expected_timestamp }
    specify { expect(metric.tags).to      eq expected_tags      }
    specify { expect(metric.values).to    eq expected_values    }
  end

  describe Logsnarf::Decoder::HerokuDynoLoad do
    let(:fixture) { "heroku_dyno_load" }

    let(:expected_name)      { "heroku_dyno_load" }
    let(:expected_timestamp) { Time.utc(2019, 11, 25, 18, 28, 0, 226_738) }
    let(:expected_tags)      { { "source" => "contact_imports_worker.2" } }
    let(:expected_values)    { { "load_avg_1m" => 0.0, "load_avg_5m" => 0.0, "load_avg_15m" => 0.0 } }

    include_examples "Decoder"
  end

  describe Logsnarf::Decoder::HerokuDynoMemory do
    let(:fixture) { "heroku_dyno_memory" }

    let(:expected_name)      { "heroku_dyno_memory" }
    let(:expected_timestamp) { Time.utc(2019, 11, 25, 18, 29, 56, 629_470) }
    let(:expected_tags)      { { "source" => "worker.2" } }
    let(:expected_values)    {
      { "memory_cache_mb" => 6.6,
        "memory_pgpgin_pages" => 153_108.0,
        "memory_pgpgout_pages" => 65_686.0,
        "memory_quota_mb" => 1024.0,
        "memory_rss_mb" => 334.89,
        "memory_swap_mb" => 0.0,
        "memory_total_mb" => 341.49 }
    }

    include_examples "Decoder"
  end

  describe Logsnarf::Decoder::HerokuPostgres do
    let(:fixture) { "heroku_postgres" }

    let(:expected_name)      { "heroku_postgres" }
    let(:expected_timestamp) { Time.utc(2019, 11, 25, 18, 28, 54) }
    let(:expected_tags)      { { "source" => "HEROKU_POSTGRESQL_GREEN", "addon" => "postgresql-triangular-70792" } }
    let(:expected_values)    {
      { "active-connections" => 12.0,
        "db_size_bytes" => 194_056_863.0,
        "index-cache-hit-rate" => 0.99996,
        "load-avg-15m" => 0.0,
        "load-avg-1m" => 0.0,
        "load-avg-5m" => 0.0,
        "memory-cached_kb" => 2_497_528.0,
        "memory-free_kb" => 12_716_940.0,
        "memory-postgres_kb" => 51_036.0,
        "memory-total_kb" => 15_657_100.0,
        "read-iops" => 0.0,
        "table-cache-hit-rate" => 0.99986,
        "tables" => 57.0,
        "waiting-connections" => 0.0,
        "write-iops" => 0.067227, }
    }

    include_examples "Decoder"
  end

  describe Logsnarf::Decoder::HerokuRedis do
    let(:fixture) { "heroku_redis" }

    let(:expected_name)      { "heroku_redis" }
    let(:expected_timestamp) { Time.utc(2019, 11, 25, 18, 29, 19) }
    let(:expected_tags)      { { "addon" => "redis-regular-64666" } }
    let(:expected_values)    {
      { "active-connections" => 18.0,
        "evicted-keys" => 0.0,
        "hit-rate" => 0.97585,
        "load-avg-15m" => 0.455,
        "load-avg-1m" => 0.0,
        "load-avg-5m" => 0.47,
        "memory-cached_kb" => 4_205_788.0,
        "memory-free_kb" => 8_642_236.0,
        "memory-redis_bytes" => 3_045_976.0,
        "memory-total_kb" => 15_664_216.0,
        "read-iops" => 0.0,
        "write-iops" => 22.552, }
    }

    include_examples "Decoder"
  end
end
