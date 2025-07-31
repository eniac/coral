pub mod log {
    use csv::Writer;
    use dashmap::DashMap;
    use std::{borrow::Cow, fmt::Display};
    use std::fs::OpenOptions;
    use std::io::{Write, Result};
    use std::path::Path;
    use std::time::{Duration, Instant};
    use std::sync::LazyLock;

    pub static TIMER: LazyLock<DashMap<TimerKey, Time>> = LazyLock::new(DashMap::new);
    pub static R1CS: LazyLock<DashMap<R1CSKey, usize>> = LazyLock::new(DashMap::new);
    pub static SPACE: LazyLock<DashMap<SpaceKey, usize>> = LazyLock::new(DashMap::new);

    #[derive(PartialEq, Eq, Debug, Hash, Clone, Copy)]
    pub enum Component {
        Compiler,
        Prover,
        Solver,
        Verifier,
        CommitmentGen,
        Generator,
    }

    impl Display for Component {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Component::Compiler => write!(f, "C"),
                Component::Prover => write!(f, "P"),
                Component::Solver => write!(f, "S"),
                Component::Generator => write!(f, "G"),
                Component::Verifier => write!(f, "V"),
                Component::CommitmentGen => write!(f, "CG"),
            }
        }
    }

    #[derive(PartialEq, Eq, Debug, Hash, Clone)]
    pub enum TestType {
        Constraints,
        Runtime,
        Size,
    }

    impl Display for TestType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                TestType::Constraints => write!(f, "NOC"),
                TestType::Runtime => write!(f, "R"),
                TestType::Size => write!(f, "S"),
            }
        }
    }

    pub type SpaceKey = (Component, Cow<'static, str>);
    pub type R1CSKey = (Component, Cow<'static, str>);
    pub type TimerKey = (Component, Cow<'static, str>);
    
    #[derive(PartialEq, Eq, Debug, Clone)]
    pub enum Time {
        Started(Instant),
        Finished(Duration),
        Restarted(Duration, Instant),
    }

    use Time::*;
    pub fn r1cs(comp: Component, test: impl Into<Cow<'static, str>>, num_constraints: usize) {
        let key = (comp, test.into());
        if R1CS.contains_key(&key) {
            panic!("Trying to write multiple r1cs for same test")
        } else {
            R1CS.insert(
                key,
                num_constraints,
            );
        }
    }

    pub fn space(comp: Component, test: impl Into<Cow<'static, str>>, sz_bytes: usize) {
        let key = (comp, test.into());
        if SPACE.contains_key(&key) {
            panic!("Trying to write multiple sizes for same test")
        } else {
            SPACE.insert(key, sz_bytes);
        }
    }

    pub fn tic(comp: Component, test: impl Into<Cow<'static, str>>) {
        let key = (comp, test.into());
        if TIMER.contains_key(&key) {
            TIMER.alter(
                &key,
                |_, v| match v {
                    Started(start_time) => Finished(start_time.elapsed()),
                    Finished(duration) => Restarted(duration, Instant::now()),
                    Restarted(duration, start_time) => Finished(duration + start_time.elapsed()),
                },
            );
        } else {
            TIMER.insert(
                key,
                Started(Instant::now()),
            );
        }
    }

    pub fn stop(comp: Component, test: impl Into<Cow<'static, str>>) {
        TIMER.alter(
            &(comp, test.into()),
            |_, v| match v {
                Started(start_time) => Finished(start_time.elapsed()),
                Finished(duration) => Finished(duration),
                Restarted(duration, start_time) => Finished(duration + start_time.elapsed()),
            },
        );
    }

    pub fn clear_finished() {
        TIMER.retain(|_, v| match v {
            Started(_) | Restarted(_, _) => true,
            Finished(_) => false,
        })
    }

    pub fn write_csv(out: &str) -> Result<()> {
        println!("Writing timer data to {out}");
        let mut write_header = true;
        if Path::new(&out).exists() {
            write_header = false;
        }
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(out)
            .unwrap();
        let mut wtr = Writer::from_writer(file);

        if write_header {
            wtr.write_record(&["type", "component", "test", "value", "metric_type"])?;
        }

        let mut c_buf = Vec::new();
        let mut value_buf = Vec::new();
        let timer_test_type = TestType::Runtime.to_string();
        let size_test_type = TestType::Size.to_string();
        let r1cs_test_type = TestType::Constraints.to_string();
        for v in TIMER.iter() {
            let ((c, test), value) = v.pair();
            if let Finished(duration) = value {
                write!(c_buf, "{c}").unwrap();
                write!(value_buf, "{:?}", duration.as_micros()).unwrap();
                wtr.write_record(&[
                    timer_test_type.as_bytes(),
                    &c_buf,
                    test.as_bytes(),
                    value_buf.as_slice(),
                    "μs".as_bytes(),
                ])?;
            }
            c_buf.clear();
            value_buf.clear();
        }
        //println!("times");
        for v in R1CS.iter() {
            let ((c, test), value) = v.pair();
            write!(c_buf, "{c}").unwrap();
            write!(value_buf, "{:?}", value).unwrap();
            wtr.write_record(&[
                r1cs_test_type.as_bytes(),
                &c_buf,
                test.as_bytes(),
                &value_buf,
                "constraints".as_bytes(),
            ])?;
            c_buf.clear();
            value_buf.clear();
        }
        //println!("r1cs");

        for v in SPACE.iter() {
            let ((c, test), value) = v.pair();
            write!(c_buf, "{c}").unwrap();
            write!(value_buf, "{:?}", value).unwrap();
            wtr.write_record(&[
                size_test_type.as_bytes(),
                &c_buf,
                test.as_bytes(),
                &value_buf,
                "bytes".as_bytes(),
            ])?;
            c_buf.clear();
            value_buf.clear();
        }
        //println!("space");
        wtr.flush()?;
        clear_finished();
        R1CS.clear();
        SPACE.clear();
        Ok(())
    }
}