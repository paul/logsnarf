# frozen_string_literal: true

require "bundler/setup"
require "date"
require "time"
require "logsnarf"

ENV["TZ"] = "UTC"

samples = []

Dir["samples/*.log"].each.with_index do |file, idx|
  data = File.read(file)
  new_data = String.new
  data.each_line do |line|
    timestamp = line.split(" ", 4)[2]
    time = DateTime.rfc3339(timestamp).to_time
    now = Time.now - idx
    new_timestamp = if time.subsec == 0
                      now.strftime("%Y-%m-%dT%H:%M:%S%:z")
                    else
                      now.strftime("%Y-%m-%dT%H:%M:%S.%6N%:z")
                    end

    new_data << line.gsub(timestamp, new_timestamp)
  end

  metrics = []
  Logsnarf::Parser.new(new_data).each_metric do |log_data|
    decoder = Logsnarf::DECODERS.detect { |dec| dec.valid?(log_data) }&.new(log_data)
    metrics << decoder if decoder
  end

  samples << metrics unless metrics.empty?
end

creds = {
  "credentials" => {
    "influxdb_url" => "https://tesseract-stg:Oohan4zoong8ahp8eaneDee0@calvinklein-5713f949.influxcloud.net:8086/tesseract_stg"

  },
  "url" => "https://calvinklein-5713f949.influxcloud.net:8086/write?db=tesseract_stg&precision=u"
}
@raw_headers = [["Authorization", "Basic dGVzc2VyYWN0LXN0ZzpPb2hhbjR6b29uZzhhaHA4ZWFuZURlZTA="]]
# adapter = Logsnarf::Adapter::InfluxdbV1.new(creds, logger: Async.logger, instrumenter: nil)

# samples.values_at(5).each do |metrics|
#   ap metrics
#   adapter.write_metrics(metrics) unless metrics.empty?
# end

def setup(creds, logger:, instrumenter:)
  @creds = creds
  # @logger, @instrumenter = logger.with(name: "influxdb_v1 #{creds['name']}"), instrumenter
  @uri = URI.parse(@creds.dig("credentials", "influxdb_url"))
  @internet = Async::HTTP::Internet.new
  # at_exit { stop }
end

def write_metrics(metrics)
  # metrics = Array(metrics)
  Async do
    body = Logsnarf::Encoders::Influxdb.new(metrics).call
    ap headers: headers, raw_headers: @raw_headers
    response = @internet.post(@creds["url"], headers, Async::HTTP::Body::Buffered.wrap(body))
    ap response.status
  end
end

def url
  @url ||= begin
    query = URI.encode_www_form(
      db: @uri.path.split("/").last,
      precision: "u"
    )

    builder = (@uri.scheme == "https" ? URI::HTTPS : URI::HTTP)
    builder.build(host: @uri.host, port: @uri.port, path: "/write", query: query).to_s
  end
end

def headers
  @headers ||= begin
    headers = []
    headers << ["Authorization", "Basic #{Base64.strict_encode64(@uri.userinfo)}"] if @uri.userinfo
    headers
  end
end

setup(creds, logger: Async.logger, instrumenter: nil)
samples.values_at(5).each do |metrics|
  write_metrics(metrics)
end

@internet.close
