
require "benchmark"

STRING = "356 <134>1 2021-10-22T23:55:19.697631+00:00 host heroku events_worker.3 - source=events_worker.3 dyno=heroku.97268060.74b6a184-8289-43a1-81cb-6f140bd0cb81 sample#memory_total=454.37MB sample#memory_rss=449.62MB sample#memory_cache=4.75MB sample#memory_swap=0.00MB sample#memory_pgpgin=356280pages sample#memory_pgpgout=239961pages sample#memory_quota=512.00MB"
SLICE = STRING.to_slice

def itr_each(char, str)
  SLICE.each_with_index do |c, i|
    if c == char.ord
      return {str[0, i], i+1}
    end
  end
  raise "Unexpected EOS"
end

def itr_while(char, str)
  i = 0
  while i < str.size 
    if str[i] == char.ord 
      return {str[0, i], i+1}
    else
      i += 1
    end
  end
  raise "Unexpected EOS"
end

# @[AlwaysInline]
def itr_inline_each(char, str)
  SLICE.each_with_index do |c, i|
    if c == char.ord
      return {str[0, i], i+1}
    end
  end
  raise "Unexpected EOS"
end

HASH = '#'
Benchmark.ips() do |x|
  x.report("each")        { itr_each('#', SLICE) }
  x.report("each2")       { itr_each(HASH, SLICE) }
  # x.report("while")       { itr_while('#', SLICE) }
  # x.report("inline each") { itr_inline_each('#', SLICE) }
end


