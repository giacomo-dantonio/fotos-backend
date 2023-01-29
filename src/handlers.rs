use crate::{APPNAME, AppConf};

// FIXME: get the folder from the request path
pub async fn list_folder() -> String {
    let cfg : AppConf = confy::load(APPNAME, None).unwrap();

    // FIXME: return the children

    cfg.root
}
