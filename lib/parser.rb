# frozen_string_literal: true

require "strscan"

LogData = Struct.new(:line, :timestamp, :hostname, :appname, :procid, :msgid, :pairs, keyword_init: true)

class Parser
  NILVAL = "-"
  ASCII = "[\u{21}-\u{7e}]"

  SIZE = /\d+/
  PRI = /<\d+>/
  VER = /\d*/
  TIMESTAMP = /\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})/
  HOSTNAME = /[#{ASCII}]{1,255}/
  APPNAME = /[#{ASCII}]{1,48}/
  PROCID = /[#{ASCII}]{1,128}/
  MSGID = /[#{ASCII}]{1,32}/

  SD = /- / # Heroku doesn't have structured data, and sometimes doesn't include the NILVAL

  KEY = /[^\s=]+/
  VALUE = /[^\s]+/
  KV_DELIM = /=/

  NEWLINE = "\n"
  SP = /\s+/

  attr_reader :scanner

  def parse(io)
    scanner = StringScanner.new("")
    buffer = String.new
    lines, bytes = 0, 0
    while chunk = io.gets
      bytes += chunk.size
      buffer.concat(chunk)
      while idx = buffer.index(NEWLINE)
        line = buffer.slice!(0, idx + 1)
        lines += 1

        scanner.string = line
        yield parse_line(scanner)
      end
    end
    [lines, bytes]
  end

  private

  def parse_line(scanner)
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

    LogData.new(line:, timestamp:, hostname:, appname:, procid:, msgid:, pairs:) unless pairs.empty?
  end
end
