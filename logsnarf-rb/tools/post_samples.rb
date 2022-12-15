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

  samples << new_data
end

require "async"
require "async/http/internet"

ap samples.size

url = "https://localhost:9292/ingress/302da530a504cd7cd0661e54e8a9b4da"
# url = "https://do.logsnarf.com/ingress/302da530a504cd7cd0661e54e8a9b4da"
headers = []

@internet = Async::HTTP::Internet.new

Async do
  samples.each do |data|
    response = @internet.post(url, headers, Async::HTTP::Body::Buffered.wrap(data))
    next if response.status == 204

    ap(status: response.status,
       headers: response.headers.fields,
       body: response.read,
       url: url,
       request: data,
       time: data.each_line.map { |l| Time.at(l.split(" ").last.to_f / 1_000_000) },
       now: Time.now)
  end
ensure
  @internet.close
end
