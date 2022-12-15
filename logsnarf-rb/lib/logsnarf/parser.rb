# frozen_string_literal: true

require "strscan"
module Logsnarf
  LogData = Struct.new(:line, :timestamp, :hostname, :appname, :procid, :msgid, :pairs, keyword_init: true)

  class Parser
    NILVAL = "-"
    ASCII = "[\u{21}-\u{7e}]"

    SIZE = /\d+/.freeze
    PRI = /\<\d+\>/.freeze
    VER = /\d*/.freeze
    TIMESTAMP = /\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})/.freeze
    HOSTNAME = /[#{ASCII}]{1,255}/.freeze
    APPNAME = /[#{ASCII}]{1,48}/.freeze
    PROCID = /[#{ASCII}]{1,128}/.freeze
    MSGID = /[#{ASCII}]{1,32}/.freeze

    SD = /- /.freeze # Heroku doesn't have structured data, and sometimes doesn't include the NILVAL

    KEY = /[^\s=]+/.freeze
    VALUE = /[^\s]+/.freeze
    KV_DELIM = /=/.freeze

    NEWLINE = "\n"
    SP = /\s+/.freeze

    attr_reader :scanner

    def initialize(text)
      @scanner = StringScanner.new(text)
    end

    def each_metric
      until scanner.eos?
        result = parse_line
        yield result if result
      end
    end

    def first_line
      parse_line
    end

    private

    def parse_line
      line = scanner.rest.split("\n", 2).first

      scanner.scan(SIZE)
      scanner.skip(SP)

      scanner.scan(PRI)
      scanner.scan(VER)
      scanner.skip(SP)

      timestamp = scanner.scan(TIMESTAMP)
      scanner.skip(SP)

      hostname = scanner.scan(HOSTNAME)
      scanner.skip(SP)

      appname = scanner.scan(APPNAME)
      scanner.skip(SP)

      procid = scanner.scan(PROCID)
      scanner.skip(SP)

      msgid = scanner.scan(MSGID)
      scanner.skip(SP)

      scanner.skip(SD)

      pairs = {
        "hostname" => hostname,
        "appname" => appname,
        "procid" => procid,
        "msgid" => msgid
      }
      last_pos = nil
      loop do
        key = scanner.scan(KEY)
        scanner.skip(KV_DELIM)
        value = scanner.scan(VALUE)
        # scanner.skip(WHITESPACE)
        pairs[key] = value if key && value

        break if scanner.getch == NEWLINE || last_pos == scanner.pos

        last_pos = scanner.pos
      end

      LogData.new(line: line, timestamp: timestamp, hostname: hostname, appname: appname, procid: procid, msgid: msgid, pairs: pairs) unless pairs.empty?
    end
  end
end
