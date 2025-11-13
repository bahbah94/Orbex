--- Trades table: Stores executed trades from TradeExecuted Event
CREATE TABLE IF NOT EXISTS trades (

    trade_id BIGINT NOT NULL,

    block_number BIGINT NOT NULL,

    buyer TEXT NOT NULL,
    seller TEXT NOT NULL,

    buy_order_id BIGINT NOT NULL,
    sell_order_id BIGINT NOT NULL,

    -- Trade details
    price NUMERIC(38, 0) NOT NULL,  -- u128 from on-chain
    quantity NUMERIC(38, 0) NOT NULL,  -- u128 from on-chain
    value NUMERIC(38, 0) NOT NULL,  -- price * quantity
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


