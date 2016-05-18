extern crate getopts;
extern crate hyper;

use getopts::Options;
use std::env;
use hyper::Client;
use std::time::{SystemTime, Duration};
use std::thread;
use std::sync::mpsc;
use hyper::method::Method;
use hyper::header::ContentType;
use hyper::mime::{Mime, TopLevel, SubLevel};

fn measure_uri_mutl(method: Method, url: String, times: i64, body: Option<String>) {
    let thread_count = 4;
    let ave = times / thread_count;
    let total_works = ave * thread_count;
    let (tx, rx) = mpsc::channel();

    for _ in 0..thread_count {
        let url = url.clone();
        let works = ave.clone();
        let tx = tx.clone();
        let method = method.clone();
        let body = body.clone();
        thread::spawn(move || {
            let cost = match method {
                Method::Get => get_uri(url, works),
                Method::Post => {
                    post_uri(url, body.unwrap(), works)
                },
                _ => 0f64,
            };
            tx.send(cost).unwrap();
        });
    }

    let mut total_cost = 0.0;
    for _ in 0..thread_count {
        total_cost += rx.recv().unwrap();
    }
    let average_cost = total_cost / total_works as f64;
    println!("get {} {} times,\n total cost {} ms",
             url,
             total_works,
             total_cost);
    println!("average cost {} ms", average_cost);
}

fn measure_work<F>(f: F) -> f64
    where F : Fn()
{
    let now = SystemTime::now();
    f();
    let duration = SystemTime::now().duration_since(now).unwrap();
    let secs = duration.as_secs() as f64;
    let msecs = duration.subsec_nanos() as f64 / 10e6;
    let total_msecs = secs * 1000.0 + msecs;
    total_msecs
}


fn get_uri(url: String, times: i64) -> f64 {
    let mut client = Client::new();
    client.set_read_timeout(Some(Duration::new(10, 0)));
    println!("get {} repeat {} times", url, times);
    let work_fn = move || {
        for _ in 0..times {
            client.get(&url).send().unwrap();
        }
    };
    measure_work(work_fn)
}

fn post_uri(url: String, body: String, times: i64) -> f64 {
    let mut client = Client::new();
    client.set_read_timeout(Some(Duration::new(10, 0)));
    println!("post {} with body {} repeat {} times",
             url, body, times);
    let work_fn = move || {
        for _ in 0..times {
            let content_type = ContentType(Mime(TopLevel::Application,
                                                SubLevel::Json, vec![]));
            let req = client.post(&url).body(&body)
                .header(content_type);
            req.send().unwrap();
        }
    };
    measure_work(work_fn)
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} URI [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflagopt("p", "post", "option, post data", "a=1&b=2");
    opts.reqopt("t", "times", "require, repeat times", "COUNT");
    opts.optflag("h", "help", "print help");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println!("{:?}\n", e);
            print_usage(&program, opts);
            return;
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let url = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        println!("No uri specify\n");
        print_usage(&program, opts);
        return;
    };

    let times = matches.opt_str("t")
                       .and_then(|s| {
                           match s.parse() {
                               Ok(s) => Some(s),
                               Err(_) => None,
                           }
                       });
    let times = match times {
        Some(t) => t,
        None => {
            println!("times is not a number");
            print_usage(&program, opts);
            return;
        }
    };

    if matches.opt_present("p") {
        let body = matches.opt_str("p").unwrap();
        measure_uri_mutl(Method::Post, url, times, Some(body));
    } else {
        measure_uri_mutl(Method::Get, url, times, None);
    }
}
