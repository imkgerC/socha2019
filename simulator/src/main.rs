extern crate argparse;
extern crate game_sdk;
extern crate logger;
extern crate logic_player;
extern crate rand;
extern crate threadpool;
extern crate time;

mod game_runner;
mod tournament;
mod xml_utils;

use argparse::{ArgumentParser, Store, StoreTrue};
use game_runner::{collect_data, collect_selfplay, run_game_wo_sending, run_single_game};
use game_sdk::logging::{Data, Winner};
use game_sdk::PlayerColor;
use logger::Logger;
use tournament::run_tournament;

use std::sync::mpsc;
use std::thread;
use threadpool::ThreadPool;

fn main() {
    let mut n = 1;
    let mut xml_enabled = false;
    let mut data_collection = false;
    let mut threads = 7;
    let mut benchmarking = false;
    let mut tournament = false;
    let mut selfplay = false;
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut n)
            .add_option(&["-n", "--number"], Store, "number of games to simulate");
        ap.refer(&mut xml_enabled)
            .add_option(&["--xml"], StoreTrue, "Generate xml replays");
        ap.refer(&mut data_collection).add_option(
            &["-c", "--collect"],
            StoreTrue,
            "Run in data-generation mode",
        );
        ap.refer(&mut threads).add_option(
            &["-t", "--threads"],
            Store,
            "Number of threads to run with, defaults to 7",
        );
        ap.refer(&mut benchmarking).add_option(
            &["-b", "--benchmark"],
            StoreTrue,
            "If set, simulation is run in optimized benchmark mode",
        );
        ap.refer(&mut tournament).add_option(
            &["-s", "--scrimmage"],
            StoreTrue,
            "If set, a championship is played",
        );
        ap.refer(&mut selfplay).add_option(
            &["-r", "--selfplay"],
            StoreTrue,
            "If set, logged self-play is performed",
        );
        ap.parse_args_or_exit();
    }

    println!("Starting with parameters:\nn:{}\nxml:{}\ncollection:{}\nthreads:{}\nbenchmark:{}\ntourney:{}\nrl:{}\n",
                n, xml_enabled, data_collection, threads, benchmarking, tournament, selfplay);

    if tournament {
        run_tournament(threads, n, xml_enabled);
        return;
    }

    // create logging thread
    let (t_log, r_log): (mpsc::Sender<Data>, mpsc::Receiver<Data>) = mpsc::channel();
    let (t_winner, r_winner): (
        mpsc::Sender<(Winner, Option<PlayerColor>)>,
        mpsc::Receiver<(Winner, Option<PlayerColor>)>,
    ) = mpsc::channel();

    let mut handle = None;
    if data_collection {
        let mut logger = Logger::new("replays/vals/".to_string());
        handle = Some(thread::spawn(move || {
            for received in r_log {
                match received {
                    Data::Step(state) => logger.add_state(&state),
                    Data::End(state) => logger.end_state(&state),
                }
            }
        }));
    } else if selfplay {
        let mut logger = Logger::new("replays/vals/".to_string());
        handle = Some(thread::spawn(move || {
            for received in r_log {
                match received {
                    Data::Step(state) => logger.add_state(&state),
                    Data::End(state) => logger.end_state(&state),
                }
            }
        }));
    }

    let pool = ThreadPool::new(threads);

    for i in 0..n {
        // different modes to run in:
        // - selfplay
        // - data_collection
        // - normal

        let t_winner = t_winner.clone();
        let index = i;
        if data_collection {
            let t_log = t_log.clone();
            pool.execute(move || {
                collect_data(index as u32, t_log, t_winner);
            });
        } else if selfplay {
            let t_log = t_log.clone();
            pool.execute(move || {
                collect_selfplay(index as u32, t_log, t_winner);
            });
        } else if !benchmarking {
            pool.execute(move || {
                run_single_game(index as u32, t_winner, xml_enabled);
            });
        } else {
            pool.execute(move || {
                run_game_wo_sending(index as u32);
            });
        }
    }
    let before = time::now(); // start timing AFTER queuing
    if !benchmarking {
        let mut n_one = 0;
        let mut n_two = 0;
        let mut n_one_red = 0;
        let mut n_two_red = 0;
        let mut n_one_blue = 0;
        let mut n_two_blue = 0;

        let digits = (n as f32).log10().floor() as usize + 1;
        for i in 0..n {
            for state in r_winner.recv() {
                match state.0 {
                    Winner::One => n_one += 1,
                    Winner::Two => n_two += 1,
                    Winner::Draw => {}
                };
                if let Some(c) = state.1 {
                    if state.0 == Winner::One {
                        match c {
                            PlayerColor::Red => n_one_red += 1,
                            PlayerColor::Blue => n_one_blue += 1,
                        };
                    } else if state.0 == Winner::Two {
                        match c {
                            PlayerColor::Red => n_two_red += 1,
                            PlayerColor::Blue => n_two_blue += 1,
                        };
                    }
                }
                let n = i as f32 + 1.;
                let draws = n - n_one as f32 - n_two as f32;
                /*let a = n_one as f32 + (draws / 2.) + 1.0;
                let b = n_two as f32 + (draws / 2.) + 1.0;
                let p_one = a / n - 1.65 * (((a * b) / (n * n * n + n * n)).sqrt());
                let p_two = b / n - 1.65 * (((a * b) / (n * n * n + n * n)).sqrt());*/

                let mu = (n_one as f32 + draws / 2.) / n;
                let stdev = f32::sqrt(
                    n_one as f32 / n * (1. - mu).powi(2)
                        + n_two as f32 / n * (0. - mu).powi(2)
                        + draws / n * (0.5 - mu).powi(2),
                ) / f32::sqrt(n);

                let elo_one = -400. * ((1. / mu) - 1.).log10();
                let elo_high = -400. * ((1. / (mu + 1.96 * stdev)) - 1.).log10();
                let elo_low = -400. * ((1. / (mu - 1.96 * stdev)) - 1.).log10();
                /*let elo_two = -400. * ((n / mu) - 1.).log10();

                let elo_one_low = -400. * ((1. / p_one) - 1.).log10();
                let elo_two_low = -400. * ((1. / p_two) - 1.).log10();
                let elo_dev = f32::max(
                    f32::abs(elo_one_low - elo_one),
                    f32::abs(elo_two_low - elo_two),
                );*/
                println!(
                    "{:digits$.}:{} +{:.} -{:.} ={:.0} | [{:+.1},{:+.1}] {:+.0} | r1 {:} b1 {:}, r2 {:} b2 {:}",
                    i + 1,
                    state.0,
                    n_one,
                    n_two,
                    draws,
                    elo_low,
                    elo_high,
                    elo_one,
                    n_one_red,
                    n_one_blue,
                    n_two_red,
                    n_two_blue,
                    digits = digits,
                );
            }
        }
        println!(
            "{}:{} r1{} b1{} r2{} b2{}",
            n_one, n_two, n_one_red, n_one_blue, n_two_red, n_two_blue
        );
    } else {
        pool.join();
    }
    let millis_taken = (time::now() - before).num_milliseconds();
    println!(
        "Average speed of {}",
        (n as f32 / millis_taken as f32) * 1000.
    );
    drop(t_log);
    if data_collection {
        handle
            .expect("Should have been initialized, ERROR")
            .join()
            .unwrap();
    }
}
