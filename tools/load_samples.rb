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

require "async"
require "async/http/internet"

# url = "https://localhost:8087/write?db=logsnarf&precision=u"
# headers = []
url = "https://calvinklein-5713f949.influxcloud.net:8086/write?db=tesseract_stg&precision=u"
headers = [["Authorization", "Basic dGVzc2VyYWN0LXN0ZzpPb2hhbjR6b29uZzhhaHA4ZWFuZURlZTA="]]

@internet = Async::HTTP::Internet.new

samples.values_at(5).each do |metrics|
  Async do
    ap metrics
    data = Logsnarf::Encoders::Influxdb.new(metrics).call
    ap data
    response = @internet.post(url, headers, Async::HTTP::Body::Buffered.wrap(data))

    next if response.status == 204

    ap(status: response.status,
       headers: response.headers.fields,
       body: response.read,
       url: endpoint.path.dup,
       request: data,
       time: data.each_line.map { |l| Time.at(l.split(" ").last.to_f / 1_000_000) },
       now: Time.now)
  end
end

@internet.close
