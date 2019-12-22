# frozen_string_literal: true

require_relative "../system/boot"
require "logsnarf/app"
require "async/debug/selector"
require "mwrap"

app = Logsnarf::App.new

envs = 2.times.map do
  Dir["samples/*.log"].map do |f|
    {
      Rack::PATH_INFO => "/ingress/e0ff2e6751893dcd7fcb7a94d4535437",
      Rack::RACK_INPUT => StringIO.new(File.read(f))
    }
  end
end.flatten

reactor = Async::Reactor.new

GC.start
Mwrap.clear
runner = reactor.run do
  envs.each do |env|
    reactor.async do
      app.call env
    end
  end

  app.stop
end
runner.wait
reactor.stop
GC.start
GC.start
GC.start
sleep 5

# Don't track allocations for this block
Mwrap.quiet do
  puts "#{envs.size} invocations"
  results = []
  Mwrap.each do |location, total, allocations, frees, _age_total, _max_lifespan|
    results << [location, ((total / allocations.to_f) * (allocations - frees)), allocations, frees]
  end
  results.sort! do |(_, growth_a), (_, growth_b)|
    growth_b <=> growth_a
  end

  results[0..20].each do |location, growth, allocations, frees|
    next if growth == 0

    puts "#{location} growth: #{growth.to_i} allocs/frees (#{allocations}/#{frees})"
  end
end
