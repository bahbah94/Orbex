--- Trades table: Stores executed trades from TradeExecuted Event
CREATE TABLE IF NOT EXISTS trades (

    trade_id BIGINT NOT NULL,

    block_number BIGINT NOT NULL,

    buyer TEXT NOT NULL,
    seller TEXT NOT NULL,

    buy_order_id BIGINT NOT NULL,
    sell_order_id BIGINT NOT NULL,

    -- Trade details
    price NUMERIC(20, 6) NOT NULL,  -- Decimal price with 6 decimal places
    quantity NUMERIC(20, 6) NOT NULL,  -- Decimal quantity with 6 decimal places
    value NUMERIC(40, 12) NOT NULL,  -- price * quantity (needs more precision)
    symbol TEXT NOT NULL,  -- Trading pair symbol (e.g., "DOT/USDT")

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Composite primary key including the partitioning column
    PRIMARY KEY (trade_id, created_at)
);


--- NOW lets create some indexes for faster accesss for different requirements
CREATE INDEX IF NOT EXISTS idx_trades_trade_id ON trades(trade_id);

CREATE INDEX IF NOT EXISTS idx_trades_buyer ON trades(buyer);
CREATE INDEX IF NOT EXISTS idx_trades_seller ON trades(seller);

--CREATE INDEX IF NOT EXISTS idx_trades_buyer_timestamp ON trades(buyer, block_timestamp DESC);
--CREATE INDEX IF NOT EXISTS idx_trades_seller_timestamp ON trades(seller, block_timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_trades_buy_order_id ON trades(buy_order_id);
CREATE INDEX IF NOT EXISTS idx_trades_sell_order_id ON trades(sell_order_id);
