# frozen_string_literal: true

require "benchmark/ips"
require "dry/monads"
require "dry/monads/do"

GC.disable

class MyTest
  include Dry::Monads[:result]
  include Dry::Monads::Do.for(:do_notation)

  def normal
    one = "one"
    two = if one
            "two"
          end
    three = if two
              nil
            end

    if three
      :not_here
    end
  end

  def monads
    one = Success("one")
    two = if one.success?
            Success("two")
          end
    three = if two.success?
              Failure("three")
            end
    if three.success?
      Success("four")
    end
  end

  def do_notation
    one = yield Success("one")
    two = yield Success("two")
    three = yield Failure("three")
    yield Success("four")
  end
end

TEST = MyTest.new

Benchmark.ips do |x|
  x.report("normal") { TEST.normal }
  x.report("monads") { TEST.monads }
  x.report("do")     { TEST.do_notation }

  x.compare!
end

__END__

$ ruby --version
ruby 3.1.2p20 (2022-04-12 revision 4491bb740a) [x86_64-linux]

$ ruby benchmarks/do_notation.rb
Warming up --------------------------------------
              normal     1.556M i/100ms
              monads    53.610k i/100ms
                  do    20.078k i/100ms
Calculating -------------------------------------
              normal     15.370M (± 1.0%) i/s -     77.782M in   5.061011s
              monads    481.398k (± 9.4%) i/s -      2.412M in   5.053443s
                  do    205.505k (± 4.4%) i/s -      1.044M in   5.089954s

Comparison:
              normal: 15370415.8 i/s
              monads:   481398.2 i/s - 31.93x  (± 0.00) slower
                  do:   205505.1 i/s - 74.79x  (± 0.00) slower
