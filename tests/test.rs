#[derive(Debug, Clone)]
struct Interval {
    start: f64,
    end: f64,
    value: f64,
}

#[derive(Debug, Clone)]
struct BpmCrotchetFunction {
    intervals: Vec<Interval>,
}

impl BpmCrotchetFunction {
    fn bpm_timet_function(&self, x: f64) -> f64 {
        if x <= 0.0 {
            return 0.0;
        }

        let mut remaining = x;
        let mut acc = 0.0;
        let n = self.intervals.len();

        if n == 0 {
            return 0.0;
        }

        for (i, it) in self.intervals.iter().enumerate() {
            if remaining <= 0.0 {
                break;
            }

            if x <= it.start {
                if i == 0 {
                    return 0.0;
                } else {
                    let prev = &self.intervals[i - 1];
                    let length = x - prev.end.max(prev.start);
                    if length > 0.0 {
                        acc += 60.0 * length / prev.value;
                    }
                    return acc;
                }
            }

            let seg_start = it.start.max(0.0);
            let seg_end = it.end.min(x);
            if seg_end > seg_start {
                let len = seg_end - seg_start;
                acc += 60.0 * len / it.value;
                remaining = x - seg_end;
                if seg_end >= x {
                    return acc;
                }
            }
        }

        let last = &self.intervals[n - 1];
        if x > last.end {
            let extra_len = x - last.end;
            acc += 60.0 * extra_len / last.value;
        }

        acc
    }
}

#[test]
fn test() {
    let intervals = vec![
        Interval {
            start: 0.0,
            end: 100.0,
            value: 100.0,
        },
        Interval {
            start: 100.0,
            end: 200.0,
            value: 200.0,
        },
        Interval {
            start: 200.0,
            end: f64::INFINITY,
            value: 200.0,
        },
    ];
    let f = BpmCrotchetFunction { intervals };

    let xs = [50.0, 150.0, 250.0];
    for &x in &xs {
        let val = f.bpm_timet_function(x);
        println!("F({}) = {}", x, val);
    }

    // 结果应为：
    // F(50)  = 60 * (50/100)  = 30
    // F(150) = 60 * (100/100 + 50/200) = 60 * (1 + 0.25) = 75
    // F(250) = 60 * (100/100 + 100/200 + 50/200) = 105
}
