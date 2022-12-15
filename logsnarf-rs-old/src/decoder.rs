use crate::log_data::LogData;

pub struct Decoder<'a> {
    matcher: &'a dyn Fn(&LogData) -> bool,
    pub name: &'a str,
    pub tag_names: &'a [&'a str],
    pub field_names: &'a [&'a str],
}

const DYNO_LOAD_DECODER: Decoder<'static> = Decoder {
    // match_field: ("appname", "heroku"),
    matcher: &{ |ld: &LogData| ld.appname == "heroku" && ld.msg.contains("load_avg_1m") },
    name: "heroku_dyno_load",
    tag_names: &["source"],
    field_names: &[
        "sample#load_avg_1m",
        "sample#load_avg_5m",
        "sample#load_avg_15m",
    ],
};

const DYNO_MEMORY_DECODER: Decoder<'static> = Decoder {
    matcher: &{ |ld: &LogData| ld.appname == "heroku" && ld.msg.contains("memory_total") },
    name: "heroku_dyno_memory",
    tag_names: &["source"],
    field_names: &[
        "sample#memory_total",
        "sample#memory_rss",
        "sample#memory_cache",
        "sample#memory_swap",
        "sample#memory_pgpgin",
        "sample#memory_pgpgout",
        "sample#memory_quota",
    ],
};

const POSTGRES_DECODER: Decoder<'static> = Decoder {
    matcher: &{ |ld: &LogData| ld.procid == "heroku-postgres" },
    name: "heroku_postgres",
    tag_names: &["addon", "source"],
    field_names: &[
        "sample#db_size",
        "sample#tables",
        "sample#active-connections",
        "sample#waiting-connections",
        "sample#index-cache-hit-rate",
        "sample#table-cache-hit-rate",
        "sample#load-avg-1m",
        "sample#load-avg-5m",
        "sample#load-avg-15m",
        "sample#read-iops",
        "sample#write-iops",
        "sample#memory-total",
        "sample#memory-free",
        "sample#memory-cached",
        "sample#memory-postgres",
    ],
};

const REDIS_DECODER: Decoder<'static> = Decoder {
    matcher: &{ |ld: &LogData| ld.procid == "heroku-redis" },
    name: "heroku_redis",
    tag_names: &["addon"],
    field_names: &[
        "sample#active-connections",
        "sample#load-avg-1m",
        "sample#load-avg-5m",
        "sample#load-avg-15m",
        "sample#read-iops",
        "sample#write-iops",
        "sample#memory-total",
        "sample#memory-free",
        "sample#memory-cached",
        "sample#memory-redis",
        "sample#hit-rate",
        "sample#evicted-keys",
    ],
};

const ROUTER_DECODER: Decoder<'static> = Decoder {
    matcher: &{ |ld: &LogData| ld.procid == "heroku-router" },
    name: "heroku_router",
    tag_names: &["method", "host", "dyno", "status", "protocol"],
    field_names: &["connect", "service", "bytes"],
};

const DECODERS: &'static [Decoder] = &[
    DYNO_LOAD_DECODER,
    DYNO_MEMORY_DECODER,
    POSTGRES_DECODER,
    REDIS_DECODER,
    ROUTER_DECODER,
];

pub fn find_decoder(ld: &LogData) -> Option<&Decoder<'_>> {
    for decoder in DECODERS {
        if (decoder.matcher)(ld) {
            return Some(decoder);
        }
    }
    None
}
