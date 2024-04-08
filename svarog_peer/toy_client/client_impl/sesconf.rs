use svarog_grpc::{Algorithm, SessionConfig};

/// No math constraints between global threshold and group thresholds.
/// All members should attend.
pub fn sesconf_keygen(algo: &Algorithm, sesman_url: &str) -> SessionConfig {
    let mut conf = SessionConfig::default();
    conf.sesman_url = sesman_url.to_string();
    conf.session_id = SESID_KEYGEN.to_owned();
    conf.algorithm = algo.clone().into();
    conf.threshold = 4;

    let mut g = Group::default();
    g.group_name = "halogen".to_owned();
    g.threshold = 2;
    let att = &mut g.member_attendance;
    att.insert("fluorine".to_owned(), true);
    att.insert("chlorine".to_owned(), true);
    att.insert("bromine".to_owned(), true);
    conf.groups.push(g);

    let mut g = Group::default();
    g.group_name = "noble_gas".to_owned();
    g.threshold = 1;
    let att = &mut g.member_attendance;
    att.insert("helium".to_owned(), true);
    att.insert("neon".to_owned(), true);
    att.insert("argon".to_owned(), true);
    conf.groups.push(g);

    conf
}

/// Thresholds are ignored.
/// If a keygen member refuses to attend,
///     he/she should be configured with `false`, instead of being absent from the map.
pub fn sesconf_sign(algo: &Algorithm, sesman_url: &str) -> SessionConfig {
    let mut conf = SessionConfig::default();
    conf.sesman_url = sesman_url.to_string();
    conf.session_id = SESID_SIGN.to_owned();
    conf.algorithm = algo.clone().into();

    let mut g = Group::default();
    g.group_name = "halogen".to_owned();
    g.threshold = 2;
    let att = &mut g.member_attendance;
    att.insert("fluorine".to_owned(), false);
    att.insert("chlorine".to_owned(), true);
    att.insert("bromine".to_owned(), true);
    conf.groups.push(g);

    let mut g = Group::default();
    g.group_name = "noble_gas".to_owned();
    g.threshold = 1;
    let att = &mut g.member_attendance;
    att.insert("helium".to_owned(), true);
    att.insert("neon".to_owned(), false);
    att.insert("argon".to_owned(), true);
    conf.groups.push(g);

    conf
}

pub const SESID_KEYGEN: &str = "a6b65314fb234a2da6b29e8036b59be6";
pub const SESID_SIGN: &str = "ba2e15797ffa4e62859155fc7fc50556";
pub const SESID_SIGN_BATCH: &str = "b285aa5a668c43608f37465eebca7232";
pub const SESID_RESHARE: &str = "c24f01d0af1f4cb4acb77fb1a8f1839b";
pub const SESID_SIGN_AFTER_RESHARE: &str = "d2d31e5fdce4445eb315d4927b8c7fb2";

pub const TX_HASHES: [&str; 3] = [
    // blake2b_256("Je ne veux pas travailler. Je ne veux pas dejeuner. Je veux seulement l'oublier, et puis je fume.")
    "0db666ad5f01d64a62e81fc3284e3b00851ccef419ad9dbc3273d75e01aad102",
    // blake2b_256("Mon nom ne vous dit rien. Vous devez ignorer. Que nous sommes voisins. Depuis le mois de mai.")
    "fb957c3e6b156d14f1d83a30dd1f00d9836267e62c91fbf5ae0575a79dba1518",
    // blake2b_256("Ma flamme et mon chagrin, sais aussi mes regrets. De ne vous avoir pas, suivi sur le quai.")
    "a39e70f5c3cc4e1bf0f7936ff1f3d6564333561a461afabb760565dfc0888cb9",
];
