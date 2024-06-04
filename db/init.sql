\c demo


CREATE TABLE ohlc(
    id SERIAL,
    start TIMESTAMPTZ,
    open BIGINT,
    high BIGINT,
    low BIGINT,
    close BIGINT,
    duration INT
);

GRANT ALL PRIVILEGES ON TABLE ohlc TO demouser;

