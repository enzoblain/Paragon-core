use chrono::{DateTime, Utc};

use crate::application::context::AppContext;
use crate::domain::entities::candle::Candle;
use crate::domain::entities::data::Data;
use crate::domain::entities::direction::Direction;
use crate::domain::entities::structures::{
    OneDStructure, OneDStructureLabel, TwoDStructure, TwoDStructureLabel,
};
use crate::domain::entities::symbol::Symbol;
use crate::domain::entities::timerange::Timerange;
use crate::domain::entities::trend::{Subtrend, Trend, QUEUE, SUBTRENDS, TRENDS};

pub async fn process_trend(ctx: &AppContext, candle: &Candle) {
    let key = (candle.symbol, candle.timerange);
    QUEUE.entry(key).or_default().push(candle.clone());

    loop {
        let datetime = match get_trend(ctx, candle).await {
            Some(dt) => dt,
            None => break,
        };

        process_queue(&key, datetime);

        let queue = match QUEUE.get(&key) {
            Some(q) => q,
            None => break,
        };

        let mut found = false;
        for candle in queue.iter() {
            if let Some(datetime) = get_trend(ctx, candle).await {
                process_queue(&key, datetime);

                found = true;

                break;
            }
        }

        if !found {
            break;
        }
    }
}

pub fn process_queue(key: &(Symbol, &'static Timerange), datetime: DateTime<Utc>) {
    if let Some(mut queue) = QUEUE.get_mut(key) {
        queue.retain(|candle| candle.timestamp > datetime);
    }
}

pub async fn get_trend(ctx: &AppContext, candle: &Candle) -> Option<DateTime<Utc>> {
    let key = (candle.symbol, candle.timerange);

    let mut datetime = None;

    if let Some(mut trend) = TRENDS.get_mut(&key) {
        if candle.direction == trend.direction {
            trend.end_time = candle.timestamp;

            if let Some(mut subtrend) = SUBTRENDS.get_mut(&key) {
                // If a subtrend exists
                // We check if we have to delete it or update it
                match subtrend.direction {
                    Direction::Bullish => {
                        if candle.close < subtrend.low {
                            // In a bullish subtrend, (and a bearish trend + candle)
                            // If the candle closes below the last relative low,
                            // The subtrend is invalidated
                            // It's a BOS
                            // And we can update the trend
                            trend.high = subtrend.high;

                            let bos = Data::OneDStructure(OneDStructure {
                                symbol: candle.symbol,
                                label: OneDStructureLabel::BOS,
                                timerange: candle.timerange,
                                timestamp: candle.timestamp,
                                price: subtrend.low,
                                direction: Direction::Bearish,
                            });
                            ctx.insert_data(&bos).await;
                            ctx.send_data(bos).await;

                            let rh = Data::OneDStructure(OneDStructure {
                                symbol: candle.symbol,
                                label: OneDStructureLabel::RH,
                                timerange: candle.timerange,
                                timestamp: subtrend.last_relative_high_datetime,
                                price: subtrend.high,
                                direction: Direction::Bearish,
                            });
                            ctx.insert_data(&rh).await;
                            ctx.send_data(rh).await;

                            let rl = Data::OneDStructure(OneDStructure {
                                symbol: candle.symbol,
                                label: OneDStructureLabel::RL,
                                timerange: candle.timerange,
                                timestamp: subtrend.last_relative_low_datetime,
                                price: subtrend.low,
                                direction: Direction::Bearish,
                            });
                            ctx.insert_data(&rl).await;
                            ctx.send_data(rl).await;

                            SUBTRENDS.remove(&key);
                        } else if candle.high > subtrend.high {
                            subtrend.high = candle.high;
                            subtrend.last_relative_high_datetime = candle.timestamp;
                            subtrend.last_relative_high = candle.high;
                        }
                    }
                    Direction::Bearish => {
                        if candle.close > subtrend.high {
                            // In a bearish subtrend, (and a bullish trend + candle)
                            // If the candle closes above the last relative high,
                            // The subtrend is invalidated
                            // It's a BOS
                            // And we can update the trend
                            trend.low = subtrend.low;

                            let bos = Data::OneDStructure(OneDStructure {
                                symbol: candle.symbol,
                                label: OneDStructureLabel::BOS,
                                timerange: candle.timerange,
                                timestamp: candle.timestamp,
                                price: subtrend.high,
                                direction: Direction::Bullish,
                            });
                            ctx.insert_data(&bos).await;
                            ctx.send_data(bos).await;

                            let rh = Data::OneDStructure(OneDStructure {
                                symbol: candle.symbol,
                                label: OneDStructureLabel::RH,
                                timerange: candle.timerange,
                                timestamp: subtrend.last_relative_high_datetime,
                                price: subtrend.high,
                                direction: Direction::Bullish,
                            });
                            ctx.insert_data(&rh).await;
                            ctx.send_data(rh).await;

                            let rl = Data::OneDStructure(OneDStructure {
                                symbol: candle.symbol,
                                label: OneDStructureLabel::RL,
                                timerange: candle.timerange,
                                timestamp: subtrend.last_relative_low_datetime,
                                price: subtrend.low,
                                direction: Direction::Bullish,
                            });
                            ctx.insert_data(&rl).await;
                            ctx.send_data(rl).await;

                            SUBTRENDS.remove(&key);
                        } else if candle.low < subtrend.low {
                            subtrend.low = candle.low;
                            subtrend.last_relative_low_datetime = candle.timestamp;
                            subtrend.last_relative_low = candle.low;
                        }
                    }
                    _ => {}
                }
            } else {
                // If no subtrend exists, we update the trend high/low
                if trend.low > candle.low {
                    trend.low = candle.low;
                    trend.low_datetime = Some(candle.timestamp);
                }

                if trend.high < candle.high {
                    trend.high = candle.high;
                    trend.high_datetime = Some(candle.timestamp);
                }
            }
        } else if let Some(subtrend) = SUBTRENDS.get(&key) {
            match subtrend.direction {
                Direction::Bullish => {
                    trend.end_time = candle.timestamp;

                    if candle.close > trend.high {
                        // If we are in a bearish trend
                        // And the candle closes above the last relative high of the trend
                        // This means that there is a reversal (Change Of Character)
                        datetime = Some(subtrend.start_time);

                        let ob = Data::TwoDStructure(TwoDStructure {
                            symbol: candle.symbol,
                            label: TwoDStructureLabel::OB,
                            timerange: candle.timerange,
                            timestamp: candle.timestamp,
                            high: subtrend.last_candle.high,
                            low: subtrend.last_candle.low,
                            direction: Direction::Bullish,
                        });
                        ctx.insert_data(&ob).await;
                        ctx.send_data(ob).await;

                        let choch = Data::OneDStructure(OneDStructure {
                            symbol: candle.symbol,
                            label: OneDStructureLabel::CHOCH,
                            timerange: candle.timerange,
                            timestamp: candle.timestamp,
                            price: subtrend.last_relative_high,
                            direction: Direction::Bullish,
                        });
                        ctx.insert_data(&choch).await;
                        ctx.send_data(choch).await;
                    }
                }
                Direction::Bearish => {
                    trend.end_time = candle.timestamp;

                    if candle.close < trend.low {
                        // If we are in a bullish trend
                        // And the candle closes below the last relative low of the trend
                        // This means that there is a reversal (Change Of Character)
                        datetime = Some(subtrend.start_time);

                        let ob = Data::TwoDStructure(TwoDStructure {
                            symbol: candle.symbol,
                            label: TwoDStructureLabel::OB,
                            timerange: candle.timerange,
                            timestamp: candle.timestamp,
                            high: subtrend.last_candle.high,
                            low: subtrend.last_candle.low,
                            direction: Direction::Bearish,
                        });
                        ctx.insert_data(&ob).await;
                        ctx.send_data(ob).await;

                        let choch = Data::OneDStructure(OneDStructure {
                            symbol: candle.symbol,
                            label: OneDStructureLabel::CHOCH,
                            timerange: candle.timerange,
                            timestamp: candle.timestamp,
                            price: subtrend.last_relative_low,
                            direction: Direction::Bearish,
                        });
                        ctx.insert_data(&choch).await;
                        ctx.send_data(choch).await;
                    }
                }
                _ => {}
            }
        } else {
            // If no subtrend exists, we create one
            // Because the candle is in the opposite direction of the trend
            let subtrend = Subtrend {
                start_time: candle.timestamp,
                direction: candle.direction,
                high: candle.high,
                low: candle.low,
                last_relative_low: candle.low,
                last_relative_high: candle.high,
                last_candle: candle.clone(),
                last_relative_low_datetime: candle.timestamp,
                last_relative_high_datetime: candle.timestamp,
            };

            SUBTRENDS.insert(key, subtrend);
        }
    } else {
        // If no trend exists, we create one
        if candle.direction == Direction::Doji {
            return None;
        }

        let trend = Trend {
            symbol: candle.symbol,
            timerange: *candle.timerange,
            start_time: candle.timestamp,
            end_time: candle.timestamp,
            high: candle.high,
            low: candle.low,
            direction: candle.direction,
            high_datetime: Some(candle.timestamp),
            low_datetime: Some(candle.timestamp),
            relative_high: Some(candle.high),
            relative_low: Some(candle.low),
        };

        TRENDS.insert(key, trend);
    }

    let trend = Data::Trend(TRENDS.get(&key).unwrap().clone());
    ctx.insert_data(&trend).await;
    ctx.send_data(trend).await;

    datetime
}
