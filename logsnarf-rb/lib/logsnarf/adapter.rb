# frozen_string_literal: true

module Logsnarf
  module Adapter
    require_relative "adapter/influxdb_v1"
    require_relative "adapter/influxdb_v2"

    ADAPTERS = {
      "influxdb_v1" => InfluxdbV1,
      "influxdb_v2" => InfluxdbV2
    }.freeze

    def [](name)
      ADAPTERS.fetch(name)
    end
    module_function :[]
  end
end
