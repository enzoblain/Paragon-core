use crate::application::context::AppContext;
use crate::domain::entities::candle::Candle;
use crate::domain::entities::data::Data;
use crate::domain::entities::direction::Direction;
use crate::domain::entities::fvg::LAST_THREE_CANDLES;
use crate::domain::entities::structures::{TwoDStructure, TwoDStructureLabel};

pub async fn process_fvg(ctx: &AppContext, candle: &Candle) {
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
                let fvg = Data::TwoDStructure(TwoDStructure {
                    symbol: candle.symbol,
                    label: TwoDStructureLabel::FVG,
                    timerange: candle.timerange,
                    timestamp: candle.timestamp,
                    high: third.close,
                    low: first.open,
                    direction: Direction::Bullish,
                });
                ctx.insert_data(&fvg).await;
                ctx.send_data(fvg).await;
            }
        }
        Direction::Bearish => {
            let first = &last_three_candles[0];
            let third = &last_three_candles[2];

            if third.close > first.open {
                let fvg = Data::TwoDStructure(TwoDStructure {
                    symbol: candle.symbol,
                    label: TwoDStructureLabel::FVG,
                    timerange: candle.timerange,
                    timestamp: candle.timestamp,
                    high: first.open,
                    low: third.close,
                    direction: Direction::Bearish,
                });
                ctx.insert_data(&fvg).await;
                ctx.send_data(fvg).await;
            }
        }
        Direction::Doji => {}
    }
}
