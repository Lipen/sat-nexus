use cadical::statik::Cadical;
use cadical::LitValue;
use sat_nexus_core::map::Map;

pub fn decode_onehot<'a, T>(var: &'a Map<T, i32>, solver: &Cadical) -> Option<&'a T> {
    var.iter().find_map(|(key, &t)| {
        if solver.val(t).unwrap() == LitValue::True {
            Some(key)
        } else {
            None
        }
    })
}
