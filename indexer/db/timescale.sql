-- Create a hypertable for the trades table
SELECT create_hypertable('trades', by_range('created_at'));
-- Create the materialized views
CREATE MATERIALIZED VIEW one_minute_candles_cs
WITH (timescaledb.continuous, timescaledb.materialized_only = false) AS
SELECT time_bucket('1 minute', created_at) AS bucket,
    symbol,
    candlestick_agg(created_at, price, quantity) AS candlestick,
    COUNT(*) AS trade_count
FROM trades
GROUP BY 1, 2 WITH NO DATA;

CREATE VIEW one_minute_candles AS
    SELECT bucket,
        symbol,
        open(candlestick) AS open,
        open_time(candlestick) AS open_time,
        high(candlestick) AS high,
        high_time(candlestick) AS high_time,
        low(candlestick) AS low,
        low_time(candlestick) AS low_time,
        close(candlestick) AS close,
        close_time(candlestick) AS close_time,
        volume(candlestick) AS volume,
        vwap(candlestick) AS vwap,
        trade_count
    FROM one_minute_candles_cs;

CREATE MATERIALIZED VIEW five_minutes_candles_cs
WITH (timescaledb.continuous, timescaledb.materialized_only = false) AS
SELECT time_bucket('5 minutes', bucket) AS bucket,
    symbol,
    rollup(candlestick) AS candlestick,
    SUM(trade_count) AS trade_count
FROM one_minute_candles_cs
GROUP BY 1, 2 WITH NO DATA;

CREATE VIEW five_minutes_candles AS
    SELECT bucket,
        symbol,
        open(candlestick) AS open,
        open_time(candlestick) AS open_time,
        high(candlestick) AS high,
        high_time(candlestick) AS high_time,
        low(candlestick) AS low,
        low_time(candlestick) AS low_time,
        close(candlestick) AS close,
        close_time(candlestick) AS close_time,
        volume(candlestick) AS volume,
        vwap(candlestick) AS vwap,
        trade_count
    FROM five_minutes_candles_cs;

CREATE MATERIALIZED VIEW fifteen_minutes_candles_cs
WITH (timescaledb.continuous, timescaledb.materialized_only = false) AS
SELECT time_bucket('15 minutes', bucket) AS bucket,
    symbol,
    rollup(candlestick) AS candlestick,
    SUM(trade_count) AS trade_count
FROM five_minutes_candles_cs
GROUP BY 1, 2 WITH NO DATA;

CREATE VIEW fifteen_minutes_candles AS
    SELECT bucket,
        symbol,
        open(candlestick) AS open,
        open_time(candlestick) AS open_time,
        high(candlestick) AS high,
        high_time(candlestick) AS high_time,
        low(candlestick) AS low,
        low_time(candlestick) AS low_time,
        close(candlestick) AS close,
        close_time(candlestick) AS close_time,
        volume(candlestick) AS volume,
        vwap(candlestick) AS vwap,
        trade_count
    FROM fifteen_minutes_candles_cs;

CREATE MATERIALIZED VIEW thirty_minutes_candles_cs
WITH (timescaledb.continuous, timescaledb.materialized_only = false) AS
SELECT time_bucket('30 minutes', bucket) AS bucket,
    symbol,
    rollup(candlestick) AS candlestick,
    SUM(trade_count) AS trade_count
FROM fifteen_minutes_candles_cs
GROUP BY 1, 2 WITH NO DATA;

CREATE VIEW thirty_minutes_candles AS
    SELECT bucket,
        symbol,
        open(candlestick) AS open,
        open_time(candlestick) AS open_time,
        high(candlestick) AS high,
        high_time(candlestick) AS high_time,
        low(candlestick) AS low,
        low_time(candlestick) AS low_time,
        close(candlestick) AS close,
        close_time(candlestick) AS close_time,
        volume(candlestick) AS volume,
        vwap(candlestick) AS vwap,
        trade_count
    FROM thirty_minutes_candles_cs;

CREATE MATERIALIZED VIEW one_hour_candles_cs
WITH (timescaledb.continuous, timescaledb.materialized_only = false) AS
SELECT time_bucket('1 hour', bucket) AS bucket,
    symbol,
    rollup(candlestick) AS candlestick,
    SUM(trade_count) AS trade_count
FROM thirty_minutes_candles_cs
GROUP BY 1, 2 WITH NO DATA;

CREATE VIEW one_hour_candles AS
    SELECT bucket,
        symbol,
        open(candlestick) AS open,
        open_time(candlestick) AS open_time,
        high(candlestick) AS high,
        high_time(candlestick) AS high_time,
        low(candlestick) AS low,
        low_time(candlestick) AS low_time,
        close(candlestick) AS close,
        close_time(candlestick) AS close_time,
        volume(candlestick) AS volume,
        vwap(candlestick) AS vwap,
        trade_count
    FROM one_hour_candles_cs;

CREATE MATERIALIZED VIEW four_hours_candles_cs
WITH (timescaledb.continuous, timescaledb.materialized_only = false) AS
SELECT time_bucket('4 hours', bucket) AS bucket,
    symbol,
    rollup(candlestick) AS candlestick,
    SUM(trade_count) AS trade_count
FROM one_hour_candles_cs
GROUP BY 1, 2 WITH NO DATA;

CREATE VIEW four_hours_candles AS
    SELECT bucket,
        symbol,
        open(candlestick) AS open,
        open_time(candlestick) AS open_time,
        high(candlestick) AS high,
        high_time(candlestick) AS high_time,
        low(candlestick) AS low,
        low_time(candlestick) AS low_time,
        close(candlestick) AS close,
        close_time(candlestick) AS close_time,
        volume(candlestick) AS volume,
        vwap(candlestick) AS vwap,
        trade_count
    FROM four_hours_candles_cs;

CREATE MATERIALIZED VIEW one_day_candles_cs
WITH (timescaledb.continuous, timescaledb.materialized_only = false) AS
SELECT time_bucket('1 day', bucket) AS bucket,
    symbol,
    rollup(candlestick) AS candlestick,
    SUM(trade_count) AS trade_count
FROM four_hours_candles_cs
GROUP BY 1, 2 WITH NO DATA;

CREATE VIEW one_day_candles AS
    SELECT bucket,
        symbol,
        open(candlestick) AS open,
        open_time(candlestick) AS open_time,
        high(candlestick) AS high,
        high_time(candlestick) AS high_time,
        low(candlestick) AS low,
        low_time(candlestick) AS low_time,
        close(candlestick) AS close,
        close_time(candlestick) AS close_time,
        volume(candlestick) AS volume,
        vwap(candlestick) AS vwap,
        trade_count
    FROM one_day_candles_cs;

CREATE MATERIALIZED VIEW one_week_candles_cs
WITH (timescaledb.continuous, timescaledb.materialized_only = false) AS
SELECT time_bucket('7 days', bucket) AS bucket,
    symbol,
    rollup(candlestick) AS candlestick,
    SUM(trade_count) AS trade_count
FROM one_day_candles_cs
GROUP BY 1, 2 WITH NO DATA;

CREATE VIEW one_week_candles AS
    SELECT bucket,
        symbol,
        open(candlestick) AS open,
        open_time(candlestick) AS open_time,
        high(candlestick) AS high,
        high_time(candlestick) AS high_time,
        low(candlestick) AS low,
        low_time(candlestick) AS low_time,
        close(candlestick) AS close,
        close_time(candlestick) AS close_time,
        volume(candlestick) AS volume,
        vwap(candlestick) AS vwap,
        trade_count
    FROM one_week_candles_cs;

CREATE MATERIALIZED VIEW one_month_candles_cs
WITH (timescaledb.continuous, timescaledb.materialized_only = false) AS
SELECT time_bucket('1 month', bucket) AS bucket,
    symbol,
    rollup(candlestick) AS candlestick,
    SUM(trade_count) AS trade_count
FROM one_day_candles_cs
GROUP BY 1, 2 WITH NO DATA;

CREATE VIEW one_month_candles AS
    SELECT bucket,
        symbol,
        open(candlestick) AS open,
        open_time(candlestick) AS open_time,
        high(candlestick) AS high,
        high_time(candlestick) AS high_time,
        low(candlestick) AS low,
        low_time(candlestick) AS low_time,
        close(candlestick) AS close,
        close_time(candlestick) AS close_time,
        volume(candlestick) AS volume,
        vwap(candlestick) AS vwap,
        trade_count
    FROM one_month_candles_cs;

-- Add continuous aggregate policies
SELECT add_continuous_aggregate_policy(
        'one_minute_candles_cs',
        start_offset => INTERVAL '1 day',
        end_offset => INTERVAL '1 minute',
        schedule_interval => INTERVAL '30 seconds'
    );
SELECT add_continuous_aggregate_policy(
        'five_minutes_candles_cs',
        start_offset => INTERVAL '1 day',
        end_offset => INTERVAL '5 minutes',
        schedule_interval => INTERVAL '2 minutes'
    );
SELECT add_continuous_aggregate_policy(
        'fifteen_minutes_candles_cs',
        start_offset => INTERVAL '1 day',
        end_offset => INTERVAL '15 minutes',
        schedule_interval => INTERVAL '7 minutes'
    );
SELECT add_continuous_aggregate_policy(
        'thirty_minutes_candles_cs',
        start_offset => INTERVAL '1 day',
        end_offset => INTERVAL '30 minutes',
        schedule_interval => INTERVAL '15 minutes'
    );
SELECT add_continuous_aggregate_policy(
        'one_hour_candles_cs',
        start_offset => INTERVAL '1 day',
        end_offset => INTERVAL '1 hour',
        schedule_interval => INTERVAL '30 minutes'
    );
SELECT add_continuous_aggregate_policy(
        'four_hours_candles_cs',
        start_offset => INTERVAL '1 day',
        end_offset => INTERVAL '4 hours',
        schedule_interval => INTERVAL '2 hours'
    );
SELECT add_continuous_aggregate_policy(
        'one_day_candles_cs',
        start_offset => INTERVAL '7 days',
        end_offset => INTERVAL '1 day',
        schedule_interval => INTERVAL '12 hours'
    );
SELECT add_continuous_aggregate_policy(
        'one_week_candles_cs',
        start_offset => INTERVAL '28 days',
        end_offset => INTERVAL '7 days',
        schedule_interval => INTERVAL '3 days'
    );
SELECT add_continuous_aggregate_policy(
        'one_month_candles_cs',
        start_offset => INTERVAL '3 months',
        end_offset => INTERVAL '1 month',
        schedule_interval => INTERVAL '14 days'
    );
