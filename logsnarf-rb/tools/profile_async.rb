# frozen_string_literal: true

require "bundler/inline"

gemfile do
  source "https://rubygems.org"
  gem "async"
  gem "async-http"
  gem "mwrap"
  gem "get_process_mem"
end

require "async/clock"
require "async/http/internet"

# How often to print a row of stats, false to disable
@print_stats = 1.0

# Make an HTTP request, do a simple async call, or do an inline call
@method = :http # :block, :async or :http

# How long to run the test, in seconds
@test_dur = 180

@reactor = Async::Reactor.new
@semaphore = Async::Semaphore.new(512)

@internet = Async::HTTP::Internet.new

def call(int)
  case @method
  when :http
    @reactor.async do
      resp = @internet.post("http://localhost:8086/write?db=logsnarf&precision=u",
                            [],
                            "loader.load,account=logsnarf\\ local\\ testing bytes=608i,lines=2i,metrics=2i,duration=0.000944 1576971049348004")
      # puts Async::Task.current.reactor.print_hierarchy
      resp.status
      resp.finish
    end

  when :async
    @reactor.async { int.to_s }

  when :block
    int.to_s

  end
end

def print_stats(now, int)
  return unless @print_stats

  if now - (@last || @start) > @print_stats
    dur = now - @start
    itrs = int / dur
    puts("%d,%0.2f,%0.2f,%0.2d" % [int, dur, GetProcessMem.new.mb, itrs])
    @last = now
  end
end

puts "i,t,mb,i/s"

GC.start
Mwrap.clear
@reactor.run do
  @start = Async::Clock.now
  int = 0
  loop do
    if int % 1000 == 0
      now = Async::Clock.now
      print_stats(now, int)
      break if now - @start > @test_dur
    end

    call int

    int += 1
    # break if int > 10
  end
end
# @reactor.wait
@reactor.stop
GC.start; GC.start; GC.start

# Don't track allocations for this block
Mwrap.quiet do
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
