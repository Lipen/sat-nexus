use cadical::statik::Cadical;
use cadical::LitValue;
use sat_nexus_core::lit::Lit;
use sat_nexus_core::map::Map;

pub fn decode_onehot<'a, T>(var: &'a Map<T, Lit>, solver: &Cadical) -> Option<&'a T> {
    var.iter().find_map(|(key, &t)| {
        if solver.val(t.get()).unwrap() == LitValue::True {
            Some(key)
        } else {
            None
        }
    })
}
