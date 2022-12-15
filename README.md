
# Logsnarf

Logsnarf is a tool that acts as a Heroku Log Drain, extracts the metric data, and pushes them to a time-series database.

As an exercise in learning a new tool or language, I use this project as a real-world example to see how the language handles these challenges. The reference implementation is in plain Ruby, which it turns out is plenty fast to handle the load on a single small server (2-core digital ocean box at ~$40/mo).

## Implementation Challenges

### Heroku Log Drain 

Heroku captures many useful metrics about your app, the dynos, and database addons, but doesn't expose them in an API for you to consume. It does however, write them to its own internal logging system.

```
241 <45>1 2019-11-25T18:28:00.226738+00:00 host heroku imports_worker.2 - source=imports_worker.2 dyno=heroku.97268060.b6e1c119-fba6-4c25-8129-ccf81cefd942 sample#load_avg_1m=0.00 sample#load_avg_5m=0.00 sample#load_avg_15m=0.00
370 <45>1 2019-11-25T18:28:15.490955+00:00 host heroku background_worker.1 - source=background_worker.1 dyno=heroku.97268060.cfb234af-179b-484d-87ef-49cf17de13ae sample#memory_total=324.41MB sample#memory_rss=317.93MB sample#memory_cache=6.48MB sample#memory_swap=0.00MB sample#memory_pgpgin=145418pages sample#memory_pgpgout=62370pages sample#memory_quota=512.00MB
636 <134>1 2019-11-25T18:28:54+00:00 host app heroku-postgres - source=HEROKU_POSTGRESQL_GREEN addon=postgresql-triangular-70792 sample#current_transaction=369961 sample#db_size=194056863bytes sample#tables=57 sample#active-connections=12 sample#waiting-connections=0 sample#index-cache-hit-rate=0.99996 sample#table-cache-hit-rate=0.99986 sample#load-avg-1m=0 sample#load-avg-5m=0 sample#load-avg-15m=0 sample#read-iops=0 sample#write-iops=0.067227 sample#tmp-disk-used=33849344 sample#tmp-disk-available=72944943104 sample#memory-total=15657100kB sample#memory-free=12716940kB sample#memory-cached=2497528kB sample#memory-postgres=51036kB415
<134>1 2019-11-25T18:29:19+00:00 host app heroku-redis - source=CACHE_STORE addon=redis-regular-64666 sample#active-connections=18 sample#load-avg-1m=0 sample#load-avg-5m=0.47 sample#load-avg-15m=0.455 sample#read-iops=0 sample#write-iops=22.552 sample#memory-total=15664216kB sample#memory-free=8642236kB sample#memory-cached=4205788kB sample#memory-redis=3045976bytes sample#hit-rate=0.97585 sample#evicted-keys=0302
<158>1 2019-11-25T18:28:00.089034+00:00 host heroku router - at=info method=GET path="/admin/sidekiq_queue_stats" host=myapp.example request_id=f24c9831-e1af-4f71-83aa-dc00a0f236fc fwd="52.90.232.237,70.132.60.79" dyno=web.1 connect=0ms service=25ms status=200 bytes=1541 protocol=https
```

You can subscribe to receive these logs at an endpoint you control via a [Heroku Log Drain](https://devcenter.heroku.com/articles/log-drains). This is all your application log though, there's no way to get just the lines with metrics. Additionally, while it can provide the logs via TCP Syslog or HTTP, the HTTP logs do not conform to [RFC-5424](https://www.rfc-editor.org/rfc/rfc5424.html).

Also, the Log Drain does not perform any buffering, so Heroku makes lots of tiny requests to the endpoint, representing a single requests to the hosted app, or a single dyno's metric data. Most often, a log drain request contains no metric data at all.

If Heroku receives a non-2xx response to the Log Drain requests, or if the responses take too long, it begins to throttle the requests it makes. It drops the missed requests on the floor and never retries, and eventually stops making requests completely if the error rate or response time remains high, resulting in lost data.

### TSDB

While Heroku makes lots of tiny requests that contain only 0 or 1 metrics, most TSDB implementations do not handle tiny requests well. Even a moderate amount of load can bring a well-provisioned InfluxDB instance down.

### Challenges

The nature of this results in the following challenges that have to be handled:

 * Since the logs do not conform to the standard, a new parser has to be written.
 * The endpoint receives many tiny requests, most of which do not contain any metric data at all, and must respond with a 200 every time, or data will be lost.
 * The metric data extracted does not conform to a single format or structure. Sometimes `load_avg` is `1` and sometimes `1.00`, sometimes memory is in `kB` or `MB` or with no units at all, etc...
 * The metric data that is extracted must be collected in a buffer, and periodically written to the TSDB.
 * Since "lots of tiny requests" is indicative of using threads to handle them, the metrics buffer collection and flushing must be threadsafe.

## Implementations

The reference implementation is in Ruby, and has seen production use for several years. The other implementations are in various states of completion, the intent being learning the language and its capabilities rather than being a production-quality application. Aside from the Ruby implementation, the Rust implementations are the most complete.

 * [logsnarf-rb](https://github.com/paul/logsnarf/tree/main/logsnarf-rb) Original reference implementation, using Ruby [async](https://github.com/socketry/async) and [falcon](https://github.com/socketry/falcon)
 * [logsnarf-rb-3](https://github.com/paul/logsnarf/tree/main/logsnarf-rb-3) Reimplementation of above, taking advantage of new async features in Ruby 3.0/3.1.
 * [logsnarf-cr](https://github.com/paul/logsnarf/tree/main/logsnarf-cr) (Oct 2021) Implementation of parser in Crystal. Showed promise in performance, but I felt it exhibited the worst parts of Ruby & Rust, so decided to focus on the Rust implementation instead.
 * [logsnarf-ex](https://github.com/paul/logsnarf/tree/main/logsnarf-ex) (Dec 2019) Implementation in Elixir. I think I struggled with getting access to and learning the low-level server libraries (Rack-equivalent), everything wanted me to work at the web-framework level.
 * [logsnarf-lambda](https://github.com/paul/logsnarf/tree/main/logsnarf-lambda) (Apr 2022) When AWS announced the "API Gateway", it seemed like a great approach. After trying it, however, it turned out to be cost-prohibitive. Since Heroku makes so many tiny requests, it would cost upwards of $300 per **DAY** to handle the log drain requests from an app with moderate usage. I tried again when AWS dropped their price, and also wrote the lambdas in Rust, but it still was much to expensive to run in production.
 * [logsnarf-go](https://github.com/paul/logsnarf/tree/main/logsnarf-go) (May 2021) Implementation in Go. I found writing the parser in Go frustrating, so also took a stab at using ragel to generate the parser for me.
 * [logsnarf-rs-old](https://github.com/paul/logsnarf/tree/main/logsnarf-rs-old) (May 2021) First implementation in Rust. Rust was going through some growing pains with ASync, and the AWS libraries had been abandoned. 
 * [logsnarf-rs](https://github.com/paul/logsnarf/tree/main/logsnarf-rs) Latest implementation in Rust. This one is still being actively worked on as time allows.





