use rayon::prelude::*;

use std::panic;
use std::sync::Mutex;

#[test]
fn collect_drop_on_unwind() {
    struct Recorddrop<'a>(i64, &'a Mutex<Vec<i64>>);

    impl<'a> Drop for Recorddrop<'a> {
        fn drop(&mut self) {
            self.1.lock().unwrap().push(self.0);
        }
    }

    let test_collect_panic = |will_panic: bool| {
        let test_vec_len = 1024;
        let panic_point = 740;

        let mut inserts = Mutex::new(Vec::new());
        let mut drops = Mutex::new(Vec::new());

        let mut a = (0..test_vec_len).collect::<Vec<_>>();
        let b = (0..test_vec_len).collect::<Vec<_>>();

        let _result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            let mut result = Vec::new();
            a.par_iter_mut()
                .zip(&b)
                .map(|(&mut a, &b)| {
                    if a > panic_point && will_panic {
                        panic!("unwinding for test");
                    }
                    let elt = a + b;
                    inserts.lock().unwrap().push(elt);
                    Recorddrop(elt, &drops)
                })
                .collect_into_vec(&mut result);
        }));

        let inserts = inserts.get_mut().unwrap();
        let drops = drops.get_mut().unwrap();
        println!("{:?}", inserts);
        println!("{:?}", drops);

        assert_eq!(inserts.len(), drops.len(), "Incorrect number of drops");
        // sort to normalize order
        inserts.sort();
        drops.sort();
        assert_eq!(inserts, drops, "Incorrect elements were dropped");
    };

    for &should_panic in &[true, false] {
        test_collect_panic(should_panic);
    }
}

