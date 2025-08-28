use crate::domain::entities::candle::Candle;
use crate::domain::entities::direction::Direction;
use crate::domain::entities::structures::LAST_THREE_CANDLES;

pub async fn process_fvg(candle: &Candle) {
    let key = (candle.symbol, candle.timerange);

    let mut last_three_candles = LAST_THREE_CANDLES.entry(key).or_default();

    last_three_candles.push(candle.clone());

    if last_three_candles.len() > 3 {
        let excess = last_three_candles.len() - 3;
        last_three_candles.drain(0..excess);
    }

    if last_three_candles.len() < 3 {
        return;
    }

    let first_direction = &last_three_candles[0].direction;
    let third_direction = &last_three_candles[2].direction;

    if first_direction != third_direction || *first_direction == Direction::Doji {
        return;
    }

    // The inequalities are reversed because the order of the candles is reversed
    // The last one is at the first place..
    match first_direction {
        Direction::Bullish => {
            let first = &last_three_candles[0];
            let third = &last_three_candles[2];

            if third.close < first.open {
                // TODO: Send FVG to websocket
                // TODO: Add FVG to database
            }
        }
        Direction::Bearish => {
            let first = &last_three_candles[0];
            let third = &last_three_candles[2];

            if third.close > first.open {
                // TODO: Send FVG to websocket
                // TODO: Add FVG to database
            }
        }
        Direction::Doji => {}
    }
}
