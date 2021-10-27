
require "benchmark"

require "../src/parser"
require "../src/fast_parser"

STRING = "356 <134>1 2021-10-22T23:55:19.697631+00:00 host heroku events_worker.3 - source=events_worker.3 dyno=heroku.97268060.74b6a184-8289-43a1-81cb-6f140bd0cb81 sample#memory_total=454.37MB sample#memory_rss=449.62MB sample#memory_cache=4.75MB sample#memory_swap=0.00MB sample#memory_pgpgin=356280pages sample#memory_pgpgout=239961pages sample#memory_quota=512.00MB"
SLICE = STRING.to_slice

puts Parser.new(STRING).parse 
puts FastParser.parse(STRING) 

Benchmark.ips do |x|

  x.report("Parser") { Parser.new(STRING).parse }
  x.report("FastParser") { FastParser.parse(STRING) }

end
